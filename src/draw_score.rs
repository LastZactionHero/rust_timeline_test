use crossterm::{
    cursor,
    style::{self},
    terminal::{self},
    QueueableCommand,
};
use std::io::{self, Write};

use crate::pitch::Pitch;

use crate::score::{NoteStateAtTime, Resolution, Score};

const NUM_PITCHES: u16 = 12;
const STAFF_COL_OFFSET: u16 = 4;
const STAFF_ROW_OFFSET: u16 = 1;

pub struct ScoreViewport {
    pub octave: u16,
    pub resolution: Resolution,
    pub bar_idx: u32,
}

pub fn draw_score(
    stdout: &mut io::Stdout,
    viewport: &ScoreViewport,
    score: &Score,
) -> io::Result<()> {
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
