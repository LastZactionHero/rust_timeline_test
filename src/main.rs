// main.rs

use crossterm::{
    event::{poll, read, Event, KeyCode},
    style::{self},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::sync::mpsc;
use std::thread;
use std::{
    io::{self, Write},
    time::Duration,
};

mod draw_score;
mod pitch;
mod player;
mod score;
mod song;

use draw_score::{draw_score, ScoreViewport};
use player::Player;
use score::Resolution;
use song::create_song;

fn main() -> io::Result<()> {
    // Get the song data from song.rs
    let score = create_song();
    let mut player = Player::create(&score, 44100);

    let mut stdout = io::stdout();
    stdout.execute(terminal::Clear(ClearType::All))?;

    let mut viewport = ScoreViewport {
        octave: 4,
        resolution: Resolution::Time1_32,
        bar_idx: 0,
    };
    draw_score(&mut stdout, &viewport, &score)?;
    stdout.queue(style::Print("\n\n"))?;
    stdout.flush()?;

    let (tx, rx) = mpsc::channel();

    let capture_input_handle = thread::spawn(move || {
        let _ = capture_input(tx);
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
                        player.toggle_playback();
                    }
                }
                draw_score(&mut stdout, &viewport, &score)?;
            }
            Err(e) => {
                eprintln!("Oh no!: {}", e);
                break;
            }
        }
    }
    capture_input_handle.join().unwrap();
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

fn capture_input(tx: mpsc::Sender<InputEvent>) -> io::Result<()> {
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
