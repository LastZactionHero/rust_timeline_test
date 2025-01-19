// song.rs

use crate::pitch::{Pitch, Tone};
use crate::score::{Note, Score};
use std::collections::HashMap;

pub fn create_song() -> Score {
    let mut notes: HashMap<u64, Note> = HashMap::new();

    notes.insert(
        1,
        Note {
            pitch: Pitch::new(Tone::B, 4),
            duration_b32: 1,
        },
    );
    notes.insert(
        0,
        Note {
            pitch: Pitch::new(Tone::A, 4),
            duration_b32: 32,
        },
    );
    notes.insert(
        0,
        Note {
            pitch: Pitch::new(Tone::E, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        8,
        Note {
            pitch: Pitch::new(Tone::D, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        16,
        Note {
            pitch: Pitch::new(Tone::C, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        24,
        Note {
            pitch: Pitch::new(Tone::D, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        32,
        Note {
            pitch: Pitch::new(Tone::E, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        40,
        Note {
            pitch: Pitch::new(Tone::E, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        48,
        Note {
            pitch: Pitch::new(Tone::E, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        64,
        Note {
            pitch: Pitch::new(Tone::D, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        72,
        Note {
            pitch: Pitch::new(Tone::D, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        80,
        Note {
            pitch: Pitch::new(Tone::D, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        96,
        Note {
            pitch: Pitch::new(Tone::E, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        104,
        Note {
            pitch: Pitch::new(Tone::G, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        112,
        Note {
            pitch: Pitch::new(Tone::G, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        128,
        Note {
            pitch: Pitch::new(Tone::E, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        136,
        Note {
            pitch: Pitch::new(Tone::D, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        144,
        Note {
            pitch: Pitch::new(Tone::C, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        152,
        Note {
            pitch: Pitch::new(Tone::D, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        160,
        Note {
            pitch: Pitch::new(Tone::E, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        168,
        Note {
            pitch: Pitch::new(Tone::E, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        176,
        Note {
            pitch: Pitch::new(Tone::E, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        184,
        Note {
            pitch: Pitch::new(Tone::E, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        192,
        Note {
            pitch: Pitch::new(Tone::D, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        200,
        Note {
            pitch: Pitch::new(Tone::D, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        208,
        Note {
            pitch: Pitch::new(Tone::E, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        216,
        Note {
            pitch: Pitch::new(Tone::D, 4),
            duration_b32: 8,
        },
    );
    notes.insert(
        224,
        Note {
            pitch: Pitch::new(Tone::C, 4),
            duration_b32: 8,
        },
    );

    Score { bpm: 80, notes }
}
