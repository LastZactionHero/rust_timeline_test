// main.rs

use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};

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
    let resolution_str = viewport.resolution.as_str();
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
    for bar_counter in viewport.bar_idx.. {
        let col = bar_counter * (bar_length + 1) as u32 + (STAFF_COL_OFFSET as u32);
        if col + 1 >= cols as u32 {
            break;
        }
        let row = NUM_PITCHES + STAFF_ROW_OFFSET + 1;
        stdout
            .queue(cursor::MoveTo(col as u16, row))?
            .queue(style::Print(format!("{}", bar_counter)))?;
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

                onset_b32 += viewport.resolution.duration_b32();
                match note_state {
                    NoteStateAtTime::None => "-",
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

    Ok(())
}

fn main() -> io::Result<()> {
    // Get the song data from song.rs
    let score = create_song();

    let mut stdout = io::stdout();
    stdout.execute(terminal::Clear(ClearType::All))?;

    let viewport = ScoreViewport {
        octave: 4,
        resolution: Resolution::Time1_4,
        bar_idx: 0,
    };
    draw_score(&mut stdout, &viewport, &score)?;
    stdout.queue(style::Print("\n\n"))?;
    stdout.flush()?;
    Ok(())
}
