// song.rs

use crate::pitch::{Pitch, Tone};
use crate::score::{Note, Score};
use std::collections::HashMap;

pub fn create_song() -> Score {
    let mut notes: HashMap<u64, Vec<Note>> = HashMap::new();

    let song_notes = vec![
        (
            1,
            Note {
                pitch: Pitch::new(Tone::B, 4),
                duration_b32: 1,
            },
        ),
        (
            0,
            Note {
                pitch: Pitch::new(Tone::A, 4),
                duration_b32: 32,
            },
        ),
        (
            0,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
            },
        ),
        (
            8,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
            },
        ),
        (
            16,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
            },
        ),
        (
            24,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
            },
        ),
        (
            32,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
            },
        ),
        (
            40,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
            },
        ),
        (
            48,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
            },
        ),
        (
            64,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
            },
        ),
        (
            72,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
            },
        ),
        (
            80,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
            },
        ),
        (
            96,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
            },
        ),
        (
            104,
            Note {
                pitch: Pitch::new(Tone::G, 4),
                duration_b32: 8,
            },
        ),
        (
            112,
            Note {
                pitch: Pitch::new(Tone::G, 4),
                duration_b32: 8,
            },
        ),
        (
            128,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
            },
        ),
        (
            136,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
            },
        ),
        (
            144,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
            },
        ),
        (
            152,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
            },
        ),
        (
            160,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
            },
        ),
        (
            168,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
            },
        ),
        (
            176,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
            },
        ),
        (
            184,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
            },
        ),
        (
            192,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
            },
        ),
        (
            200,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
            },
        ),
        (
            208,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
            },
        ),
        (
            216,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
            },
        ),
        (
            224,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
            },
        ),
    ];

    for (onset, note) in song_notes {
        notes.entry(onset).or_insert_with(Vec::new).push(note);
    }

    Score { bpm: 80, notes }
}
