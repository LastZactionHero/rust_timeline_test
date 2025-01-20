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
                onset_b32: 1,
            },
        ),
        (
            0,
            Note {
                pitch: Pitch::new(Tone::A, 4),
                duration_b32: 32,
                onset_b32: 0,
            },
        ),
        (
            0,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 0,
            },
        ),
        (
            8,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 8,
            },
        ),
        (
            16,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 16,
            },
        ),
        (
            24,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 24,
            },
        ),
        (
            32,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 32,
            },
        ),
        (
            40,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 40,
            },
        ),
        (
            48,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 48,
            },
        ),
        (
            64,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 64,
            },
        ),
        (
            72,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 72,
            },
        ),
        (
            80,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 80,
            },
        ),
        (
            96,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 96,
            },
        ),
        (
            104,
            Note {
                pitch: Pitch::new(Tone::G, 4),
                duration_b32: 8,
                onset_b32: 104,
            },
        ),
        (
            112,
            Note {
                pitch: Pitch::new(Tone::G, 4),
                duration_b32: 8,
                onset_b32: 112,
            },
        ),
        (
            128,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 128,
            },
        ),
        (
            136,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 136,
            },
        ),
        (
            144,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 144,
            },
        ),
        (
            152,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 152,
            },
        ),
        (
            160,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 160,
            },
        ),
        (
            168,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 168,
            },
        ),
        (
            176,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 176,
            },
        ),
        (
            184,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 184,
            },
        ),
        (
            192,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 192,
            },
        ),
        (
            200,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 200,
            },
        ),
        (
            208,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 208,
            },
        ),
        (
            216,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 216,
            },
        ),
        (
            224,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 224,
            },
        ),
    ];

    for (onset, mut note) in song_notes {
        note.onset_b32 = onset; // Set the onset_b32 field
        notes.entry(onset).or_insert_with(Vec::new).push(note);
    }

    Score { bpm: 80, notes }
}
