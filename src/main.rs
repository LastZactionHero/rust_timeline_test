use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};

const NUM_PITCH: u16 = 12;
enum Pitch {
    Undefined,
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

enum Resolution {
    Time1_4,
    Time1_8,
    Time1_16,
    Time1_32,
}

struct ScoreViewport {
    octave: u16,
    resolution: Resolution,
    bar_idx: u32,
}

const STAFF_COL_OFFSET: u16 = 4;
const STAFF_ROW_OFFSET: u16 = 1;

fn draw_score(stdout: &mut io::Stdout, viewport: &ScoreViewport) -> io::Result<()> {
    // Cols
    // 0-1:    Pitch
    // 2:      Octave
    // 3:      Space
    // 4:      Staff Start |
    // 5-N:    Staff - and |

    let resolution_str = match viewport.resolution {
        Resolution::Time1_4 => "1/4",
        Resolution::Time1_8 => "1/8",
        Resolution::Time1_16 => "1/16",
        Resolution::Time1_32 => "1/32",
    };

    let bar_length = match viewport.resolution {
        Resolution::Time1_4 => 4,
        Resolution::Time1_8 => 8,
        Resolution::Time1_16 => 16,
        Resolution::Time1_32 => 32,
    };

    stdout
        .queue(cursor::MoveTo(STAFF_COL_OFFSET, 0))?
        .queue(style::Print(resolution_str))?;

    for row in 0..NUM_PITCH {
        let pitch = match row {
            0 => Pitch::C,
            1 => Pitch::Cs,
            2 => Pitch::D,
            3 => Pitch::Ds,
            4 => Pitch::E,
            5 => Pitch::F,
            6 => Pitch::Fs,
            7 => Pitch::G,
            8 => Pitch::Gs,
            9 => Pitch::A,
            10 => Pitch::As,
            11 => Pitch::B,
            _ => Pitch::Undefined,
        };

        let pitch_str = match pitch {
            Pitch::Undefined => "??",
            Pitch::C => "C",
            Pitch::Cs => "C#",
            Pitch::D => "D",
            Pitch::Ds => "Ds",
            Pitch::E => "E",
            Pitch::F => "F",
            Pitch::Fs => "F#",
            Pitch::G => "G",
            Pitch::Gs => "G#",
            Pitch::A => "A",
            Pitch::As => "A#",
            Pitch::B => "B",
        };
        let pitch_str = format!("{pitch_str}{}", viewport.octave);
        stdout
            .queue(cursor::MoveTo(0, row + STAFF_ROW_OFFSET + 1))?
            .queue(style::Print(pitch_str))?;
    }

    let (cols, _rows) = terminal::size()?;
    let mut bar_counter = viewport.bar_idx;
    for col in STAFF_COL_OFFSET..cols - STAFF_COL_OFFSET - 1 {
        let bar_marker = if (col - STAFF_COL_OFFSET) % (bar_length + 1) == 0 {
            bar_counter += 1;
            format!("{}", bar_counter)
        } else {
            "-".to_string()
        };
        stdout
            .queue(cursor::MoveTo(col, STAFF_ROW_OFFSET))?
            .queue(style::Print("-"))?
            // TODO: Fix - placement on the bottom bar to accomodate numbers with digits  > 1.
            .queue(cursor::MoveTo(col, NUM_PITCH + STAFF_ROW_OFFSET + 1))?
            .queue(style::Print(bar_marker))?;
    }
    for row in 1 + STAFF_ROW_OFFSET..=NUM_PITCH + STAFF_ROW_OFFSET {
        for col in STAFF_COL_OFFSET..cols - STAFF_COL_OFFSET - 1 {
            let symbol = if (col - STAFF_COL_OFFSET) % (bar_length + 1) == 0 {
                "|"
            } else {
                "-"
            };
            stdout
                .queue(cursor::MoveTo(col, row))?
                .queue(style::Print(symbol))?;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    let viewport = ScoreViewport {
        octave: 3,
        resolution: Resolution::Time1_4,
        bar_idx: 0,
    };
    draw_score(&mut stdout, &viewport)?;
    stdout.queue(style::Print("\n\n"))?;
    stdout.flush()?;
    Ok(())
}
