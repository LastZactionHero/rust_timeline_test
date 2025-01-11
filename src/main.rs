// main.rs

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    execute,
    style::{self, Stylize},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::{
    io::{self, Write},
    ops::BitAnd,
    time::Duration,
};

mod pitch;
mod score;
mod song;

use pitch::Pitch;
use score::{Note, NoteStateAtTime, Resolution, Score};
use song::create_song;

const NUM_PITCHES: u16 = 12;
const STAFF_COL_OFFSET: u16 = 4;
const STAFF_ROW_OFFSET: u16 = 1;

struct ScoreViewport {
    octave: u16,
    resolution: Resolution,
    bar_idx: u32,
}

fn draw_score(stdout: &mut io::Stdout, viewport: &ScoreViewport, score: &Score) -> io::Result<()> {
    let resolution_str = format!("{} ", viewport.resolution.as_str());
    let bar_length = viewport.resolution.bar_length_in_beats();

    stdout
        .queue(cursor::MoveTo(STAFF_COL_OFFSET, 0))?
        .queue(style::Print(resolution_str))?;

    for row in 0..NUM_PITCHES {
        let pitch = Pitch::from_row_index(row);
        let pitch_str = format!("{}{}", pitch, viewport.octave);
        stdout
            .queue(cursor::MoveTo(0, row + STAFF_ROW_OFFSET + 1))?
            .queue(style::Print(pitch_str))?;
    }

    let (cols, _rows) = terminal::size()?;

    // Draw bar numbers
    for col in 0..cols - 1 {
        stdout
            .queue(cursor::MoveTo(
                col as u16,
                NUM_PITCHES + STAFF_ROW_OFFSET + 1,
            ))?
            .queue(style::Print(" "))?;
    }
    for bar_counter in 0.. {
        let col = bar_counter * (bar_length + 1) as u32 + (STAFF_COL_OFFSET as u32);
        if col + 1 >= cols as u32 {
            break;
        }
        let row = NUM_PITCHES + STAFF_ROW_OFFSET + 1;
        stdout
            .queue(cursor::MoveTo(col as u16, row))?
            .queue(style::Print(format!("{}", bar_counter + viewport.bar_idx)))?;
    }

    for row in 1 + STAFF_ROW_OFFSET..=NUM_PITCHES + STAFF_ROW_OFFSET {
        let mut onset_b32 = viewport.bar_idx as u64 * 32;

        for col in STAFF_COL_OFFSET..cols - STAFF_COL_OFFSET - 1 {
            let symbol = if (col - STAFF_COL_OFFSET) % (bar_length + 1) == 0 {
                "|"
            } else {
                let pitch = Pitch::from_row_index(row - STAFF_ROW_OFFSET - 1);
                let note_state = score.note_state_at_time(
                    viewport.resolution,
                    onset_b32,
                    pitch,
                    viewport.octave,
                );
                // Just keeping these here
                // □ ■░▒▓█
                // ┌ ┐ └ ┘ ─ │ ├ ┤ ┬ ┴ ┼ ═ ║ ╔ ╗ ╚ ╝ ╠ ╣ ╦ ╩ ╬
                // ├───┤
                // █───█
                onset_b32 += viewport.resolution.duration_b32();
                match note_state {
                    NoteStateAtTime::None => "-",
                    NoteStateAtTime::Complete => "█",
                    NoteStateAtTime::Enclosed => "▒",
                    NoteStateAtTime::Starting => "├",
                    NoteStateAtTime::Middle => "─",
                    NoteStateAtTime::Ending => "┤",
                }
            };
            stdout
                .queue(cursor::MoveTo(col, row))?
                .queue(style::Print(symbol))?;
        }
    }
    stdout.flush()?;

    Ok(())
}

fn main() -> io::Result<()> {
    // Get the song data from song.rs
    let score = create_song();

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
                    _ => println!("{event:?}\r"),
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
