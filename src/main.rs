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

    let viewport = ScoreViewport {
        octave: 4,
        resolution: Resolution::Time1_32,
        bar_idx: 0,
    };
    draw_score(&mut stdout, &viewport, &score)?;
    stdout.queue(style::Print("\n\n"))?;
    stdout.flush()?;

    let (tx, rx) = mpsc::channel();

    let capture_input_handle = thread::spawn(move || {
        let _ = capture_input(tx, &viewport);
    });

    loop {
        match rx.recv() {
            Ok(msg) => {
                println!("got a message!\r");
                match msg {
                    InputEvent::UpdateViewport(viewport) => {
                        draw_score(&mut stdout, &viewport, &score).unwrap();
                    }
                    InputEvent::TogglePlayback => player.toggle_playback(),
                }
                // let _ = play_note(msg);
            }
            Err(e) => {
                eprintln!("Oh no!: {}", e);
                break;
            }
        }
    }
    capture_input_handle.join().unwrap();
    // let (tx, rx) = mpsc::channel();

    // crossterm::terminal::enable_raw_mode()?;
    // loop {
    //     if poll(Duration::from_millis(500))? {
    //         if let Event::Key(event) = read()? {
    //             match event.code {
    //                 KeyCode::Char('q') => break,
    //                 KeyCode::Up => {
    //                     if viewport.octave < pitch::OCTAVE_MAX {
    //                         viewport.octave += 1;
    //                         draw_score(&mut stdout, &viewport, &score)?;
    //                     }
    //                 }
    //                 KeyCode::Down => {
    //                     if viewport.octave > pitch::OCTAVE_MIN {
    //                         viewport.octave -= 1;
    //                         draw_score(&mut stdout, &viewport, &score)?;
    //                     }
    //                 }
    //                 KeyCode::Right => {
    //                     viewport.bar_idx += 1;
    //                     draw_score(&mut stdout, &viewport, &score)?;
    //                 }
    //                 KeyCode::Left => {
    //                     if viewport.bar_idx > 0 {
    //                         viewport.bar_idx -= 1;
    //                         draw_score(&mut stdout, &viewport, &score)?;
    //                     }
    //                 }
    //                 KeyCode::Char(']') => {
    //                     viewport.resolution = viewport.resolution.next_up();
    //                     draw_score(&mut stdout, &viewport, &score)?;
    //                 }
    //                 KeyCode::Char('[') => {
    //                     viewport.resolution = viewport.resolution.next_down();
    //                     draw_score(&mut stdout, &viewport, &score)?;
    //                 }
    //                 KeyCode::Char(' ') => {
    //                     player.toggle_playback();
    //                     draw_score(&mut stdout, &viewport, &score)?;
    //                 }
    //                 _ => println!("{event:?}\r"),
    //             }
    //         }
    //     }
    // }
    // crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

enum InputEvent {
    UpdateViewport(ScoreViewport),
    TogglePlayback,
}

fn capture_input(tx: mpsc::Sender<InputEvent>, viewport: &ScoreViewport) -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    loop {
        if poll(Duration::from_millis(500))? {
            if let Event::Key(event) = read()? {
                match event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => {
                        if viewport.octave < pitch::OCTAVE_MAX {
                            tx.send(InputEvent::UpdateViewport(
                                viewport.set_octave(viewport.octave + 1),
                            ))
                            .unwrap();
                        }
                    }
                    KeyCode::Down => {
                        if viewport.octave > pitch::OCTAVE_MIN {
                            tx.send(InputEvent::UpdateViewport(
                                viewport.set_octave(viewport.octave - 1),
                            ))
                            .unwrap();
                        }
                    }
                    KeyCode::Right => {
                        tx.send(InputEvent::UpdateViewport(
                            viewport.set_bar_idx(viewport.bar_idx + 1),
                        ))
                        .unwrap();
                    }
                    KeyCode::Left => {
                        if viewport.bar_idx > 0 {
                            tx.send(InputEvent::UpdateViewport(
                                viewport.set_bar_idx(viewport.bar_idx - 1),
                            ))
                            .unwrap();
                        }
                    }
                    KeyCode::Char(']') => {
                        tx.send(InputEvent::UpdateViewport(
                            viewport.set_resolution(viewport.resolution.next_up()),
                        ))
                        .unwrap();
                    }
                    KeyCode::Char('[') => {
                        tx.send(InputEvent::UpdateViewport(
                            viewport.set_resolution(viewport.resolution.next_down()),
                        ))
                        .unwrap();
                    }
                    KeyCode::Char(' ') => tx.send(InputEvent::TogglePlayback).unwrap(),
                    _ => println!("{event:?}\r"),
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
