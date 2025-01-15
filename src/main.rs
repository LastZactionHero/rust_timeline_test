// main.rs
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossterm::{
    event::{poll, read, Event, KeyCode},
    style::{self},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::{
    io::{self, Write},
    time::Duration,
};

mod draw_score;
mod pitch;
mod player;
mod score;
mod score_player;
mod sin_wave;
mod song;

use draw_score::{draw_score, ScoreViewport};
use player::Player;
use score::{Resolution, Score};
use song::create_song;

fn main() -> io::Result<()> {
    // Get the song data from song.rs
    let score = create_song();
    // Make score 'static
    let score: &'static Score = Box::leak(Box::new(score));
    let player = Player::create(score, 44100);
    let shared_player = Arc::new(Mutex::new(player));

    let mut stdout = io::stdout();
    stdout.execute(terminal::Clear(ClearType::All))?;

    let mut viewport = ScoreViewport {
        octave: 4,
        resolution: Resolution::Time1_32,
        bar_idx: 0,
    };
    draw_score(&mut stdout, &viewport, score)?;
    stdout.queue(style::Print("\n\n"))?;
    stdout.flush()?;

    let (tx, rx) = mpsc::channel();

    let capture_input_handle = thread::spawn(move || {
        let _ = capture_input(&tx);
    });

    let audio_shared_player = Arc::clone(&shared_player);
    let audio_out_handle = thread::spawn(move || {
        let _ = audio_player(audio_shared_player);
    });

    loop {
        match rx.recv() {
            Ok(msg) => {
                println!("got a message!\r");
                match msg {
                    InputEvent::ViewerOctaveIncrease => {
                        if viewport.octave < pitch::OCTAVE_MAX {
                            viewport = viewport.set_octave(viewport.octave + 1);
                        }
                    }
                    InputEvent::ViewerOctaveDecrease => {
                        if viewport.octave > pitch::OCTAVE_MIN {
                            viewport = viewport.set_octave(viewport.octave - 1);
                        }
                    }
                    InputEvent::ViewerBarNext => {
                        viewport = viewport.set_bar_idx(viewport.bar_idx + 1);
                    }
                    InputEvent::ViewerBarPrevious => {
                        if viewport.bar_idx > 0 {
                            viewport = viewport.set_bar_idx(viewport.bar_idx - 1);
                        }
                    }
                    InputEvent::ViewerResolutionIncrease => {
                        viewport = viewport.set_resolution(viewport.resolution.next_up());
                    }
                    InputEvent::ViewerResolutionDecrease => {
                        viewport = viewport.set_resolution(viewport.resolution.next_down());
                    }
                    InputEvent::PlayerTogglePlayback => {
                        let mut player_guard = shared_player.lock().unwrap();
                        player_guard.toggle_playback();
                    }
                }
                draw_score(&mut stdout, &viewport, score)?;
            }
            Err(e) => {
                eprintln!("Oh no!: {}", e);
                break;
            }
        }
    }
    capture_input_handle.join().unwrap();
    audio_out_handle.join().unwrap();
    Ok(())
}

enum InputEvent {
    ViewerOctaveIncrease,
    ViewerOctaveDecrease,
    ViewerBarNext,
    ViewerBarPrevious,
    ViewerResolutionIncrease,
    ViewerResolutionDecrease,
    PlayerTogglePlayback,
}

fn capture_input(tx: &mpsc::Sender<InputEvent>) -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    loop {
        if poll(Duration::from_millis(500))? {
            if let Event::Key(event) = read()? {
                match event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => tx.send(InputEvent::ViewerOctaveIncrease).unwrap(),
                    KeyCode::Down => tx.send(InputEvent::ViewerOctaveDecrease).unwrap(),
                    KeyCode::Left => tx.send(InputEvent::ViewerBarPrevious).unwrap(),
                    KeyCode::Right => tx.send(InputEvent::ViewerBarNext).unwrap(),
                    KeyCode::Char('[') => tx.send(InputEvent::ViewerResolutionDecrease).unwrap(),
                    KeyCode::Char(']') => tx.send(InputEvent::ViewerResolutionIncrease).unwrap(),
                    KeyCode::Char(' ') => tx.send(InputEvent::PlayerTogglePlayback).unwrap(),
                    _ => println!("{event:?}\r"),
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn audio_player(player: Arc<Mutex<Player>>) -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("Did not find default output device");
    let config = device.default_output_config().unwrap();

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let stream_config: cpal::StreamConfig = config.into();
    let sample_rate = stream_config.sample_rate.0 as f64;
    let channels = stream_config.channels as usize;

    let player_clone = player.clone(); // Clone the Arc for the stream closure
    let stream = device.build_output_stream(
        &stream_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, player_clone.clone()) // Clone again for each call
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    // Keep the thread alive, and keep `player` alive:
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

fn write_data(output: &mut [f32], channels: usize, player: Arc<Mutex<Player>>) {
    for frame in output.chunks_mut(channels) {
        let sample = player.lock().unwrap().next().unwrap() as f32;
        for s in frame.iter_mut() {
            *s = sample;
        }
    }
}
