// main.rs

use crossterm::{
    event::{poll, read, Event, KeyCode},
    style::{self},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
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

    crossterm::terminal::enable_raw_mode()?;
    loop {
        if poll(Duration::from_millis(500))? {
            if let Event::Key(event) = read()? {
                match event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => {
                        if viewport.octave < pitch::OCTAVE_MAX {
                            viewport.octave += 1;
                            draw_score(&mut stdout, &viewport, &score)?;
                        }
                    }
                    KeyCode::Down => {
                        if viewport.octave > pitch::OCTAVE_MIN {
                            viewport.octave -= 1;
                            draw_score(&mut stdout, &viewport, &score)?;
                        }
                    }
                    KeyCode::Right => {
                        viewport.bar_idx += 1;
                        draw_score(&mut stdout, &viewport, &score)?;
                    }
                    KeyCode::Left => {
                        if viewport.bar_idx > 0 {
                            viewport.bar_idx -= 1;
                            draw_score(&mut stdout, &viewport, &score)?;
                        }
                    }
                    KeyCode::Char(']') => {
                        viewport.resolution = viewport.resolution.next_up();
                        draw_score(&mut stdout, &viewport, &score)?;
                    }
                    KeyCode::Char('[') => {
                        viewport.resolution = viewport.resolution.next_down();
                        draw_score(&mut stdout, &viewport, &score)?;
                    }
                    KeyCode::Char(' ') => {
                        player.toggle_playback();
                        draw_score(&mut stdout, &viewport, &score)?;
                    }
                    _ => println!("{event:?}\r"),
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
