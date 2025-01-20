// song.rs

use crate::pitch::{Pitch, Tone};
use crate::score::{Note, Score};
use std::collections::HashMap;

pub fn create_song() -> Score {
    let mut notes: HashMap<u64, Vec<Note>> = HashMap::new();

    let song_notes = vec![
        // Bar 1
        (
            0,
            Note {
                pitch: Pitch::new(Tone::F, 3),
                duration_b32: 8,
                onset_b32: 0,
            },
        ),
        (
            0,
            Note {
                pitch: Pitch::new(Tone::A, 3),
                duration_b32: 8,
                onset_b32: 0,
            },
        ),
        (
            8,
            Note {
                pitch: Pitch::new(Tone::F, 3),
                duration_b32: 8,
                onset_b32: 8,
            },
        ),
        (
            8,
            Note {
                pitch: Pitch::new(Tone::A, 3),
                duration_b32: 8,
                onset_b32: 8,
            },
        ),
        (
            16,
            Note {
                pitch: Pitch::new(Tone::As, 3),
                duration_b32: 8,
                onset_b32: 16,
            },
        ),
        (
            16,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 16,
            },
        ),
        (
            24,
            Note {
                pitch: Pitch::new(Tone::As, 3),
                duration_b32: 8,
                onset_b32: 24,
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
        // Bar 2
        (
            32,
            Note {
                pitch: Pitch::new(Tone::F, 3),
                duration_b32: 8,
                onset_b32: 32,
            },
        ),
        (
            32,
            Note {
                pitch: Pitch::new(Tone::A, 3),
                duration_b32: 8,
                onset_b32: 32,
            },
        ),
        (
            40,
            Note {
                pitch: Pitch::new(Tone::F, 3),
                duration_b32: 8,
                onset_b32: 40,
            },
        ),
        (
            40,
            Note {
                pitch: Pitch::new(Tone::A, 3),
                duration_b32: 8,
                onset_b32: 40,
            },
        ),
        (
            48,
            Note {
                pitch: Pitch::new(Tone::As, 3),
                duration_b32: 8,
                onset_b32: 48,
            },
        ),
        (
            48,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 48,
            },
        ),
        (
            56,
            Note {
                pitch: Pitch::new(Tone::As, 3),
                duration_b32: 8,
                onset_b32: 56,
            },
        ),
        (
            56,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 56,
            },
        ),
        // Bar 3
        (
            64,
            Note {
                pitch: Pitch::new(Tone::G, 3),
                duration_b32: 8,
                onset_b32: 64,
            },
        ),
        (
            64,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 64,
            },
        ),
        (
            72,
            Note {
                pitch: Pitch::new(Tone::G, 3),
                duration_b32: 8,
                onset_b32: 72,
            },
        ),
        (
            72,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 72,
            },
        ),
        (
            80,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 80,
            },
        ),
        (
            80,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 80,
            },
        ),
        (
            88,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 88,
            },
        ),
        (
            88,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 88,
            },
        ),
        // Bar 4
        (
            96,
            Note {
                pitch: Pitch::new(Tone::G, 3),
                duration_b32: 8,
                onset_b32: 96,
            },
        ),
        (
            96,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 96,
            },
        ),
        (
            104,
            Note {
                pitch: Pitch::new(Tone::G, 3),
                duration_b32: 8,
                onset_b32: 104,
            },
        ),
        (
            104,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 104,
            },
        ),
        (
            112,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 112,
            },
        ),
        (
            112,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 112,
            },
        ),
        (
            120,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 120,
            },
        ),
        (
            120,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 120,
            },
        ),
        // Bar 5 (Repeat Bar 1)
        (
            128,
            Note {
                pitch: Pitch::new(Tone::F, 3),
                duration_b32: 8,
                onset_b32: 128,
            },
        ),
        (
            128,
            Note {
                pitch: Pitch::new(Tone::A, 3),
                duration_b32: 8,
                onset_b32: 128,
            },
        ),
        (
            136,
            Note {
                pitch: Pitch::new(Tone::F, 3),
                duration_b32: 8,
                onset_b32: 136,
            },
        ),
        (
            136,
            Note {
                pitch: Pitch::new(Tone::A, 3),
                duration_b32: 8,
                onset_b32: 136,
            },
        ),
        (
            144,
            Note {
                pitch: Pitch::new(Tone::As, 3),
                duration_b32: 8,
                onset_b32: 144,
            },
        ),
        (
            144,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 144,
            },
        ),
        (
            152,
            Note {
                pitch: Pitch::new(Tone::As, 3),
                duration_b32: 8,
                onset_b32: 152,
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
        // Bar 6 (Repeat Bar 2)
        (
            160,
            Note {
                pitch: Pitch::new(Tone::F, 3),
                duration_b32: 8,
                onset_b32: 160,
            },
        ),
        (
            160,
            Note {
                pitch: Pitch::new(Tone::A, 3),
                duration_b32: 8,
                onset_b32: 160,
            },
        ),
        (
            168,
            Note {
                pitch: Pitch::new(Tone::F, 3),
                duration_b32: 8,
                onset_b32: 168,
            },
        ),
        (
            168,
            Note {
                pitch: Pitch::new(Tone::A, 3),
                duration_b32: 8,
                onset_b32: 168,
            },
        ),
        (
            176,
            Note {
                pitch: Pitch::new(Tone::As, 3),
                duration_b32: 8,
                onset_b32: 176,
            },
        ),
        (
            176,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 176,
            },
        ),
        (
            184,
            Note {
                pitch: Pitch::new(Tone::As, 3),
                duration_b32: 8,
                onset_b32: 184,
            },
        ),
        (
            184,
            Note {
                pitch: Pitch::new(Tone::D, 4),
                duration_b32: 8,
                onset_b32: 184,
            },
        ),
        // Bar 7 (Repeat Bar 3)
        (
            192,
            Note {
                pitch: Pitch::new(Tone::G, 3),
                duration_b32: 8,
                onset_b32: 192,
            },
        ),
        (
            192,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 192,
            },
        ),
        (
            200,
            Note {
                pitch: Pitch::new(Tone::G, 3),
                duration_b32: 8,
                onset_b32: 200,
            },
        ),
        (
            200,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 200,
            },
        ),
        (
            208,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 208,
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
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 216,
            },
        ),
        (
            216,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 216,
            },
        ),
        // Bar 8 (Repeat Bar 4)
        (
            224,
            Note {
                pitch: Pitch::new(Tone::G, 3),
                duration_b32: 8,
                onset_b32: 224,
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
        (
            232,
            Note {
                pitch: Pitch::new(Tone::G, 3),
                duration_b32: 8,
                onset_b32: 232,
            },
        ),
        (
            232,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 232,
            },
        ),
        (
            240,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 240,
            },
        ),
        (
            240,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 240,
            },
        ),
        (
            248,
            Note {
                pitch: Pitch::new(Tone::C, 4),
                duration_b32: 8,
                onset_b32: 248,
            },
        ),
        (
            248,
            Note {
                pitch: Pitch::new(Tone::E, 4),
                duration_b32: 8,
                onset_b32: 248,
            },
        ),
    ];

    for (onset, mut note) in song_notes {
        note.onset_b32 = onset; // Ensure the onset_b32 field is set
        notes.entry(onset).or_insert_with(Vec::new).push(note);
    }

    // Adjust BPM to match the original song's tempo
    Score { bpm: 119, notes }
}
