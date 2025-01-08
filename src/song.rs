// song.rs

use crate::pitch::Pitch;
use crate::score::{Note, Score};

pub fn create_song() -> Score {
    Score {
        bpm: 80,
        notes: vec![
            Note {
                pitch: Pitch::B,
                octave: 4,
                duration_b32: 1,
                onset_b32: 1,
            },
            Note {
                pitch: Pitch::A,
                octave: 4,
                duration_b32: 32,
                onset_b32: 0,
            },
            Note {
                pitch: Pitch::E,
                octave: 4,
                duration_b32: 8,
                onset_b32: 0,
            },
            Note {
                pitch: Pitch::D,
                octave: 4,
                duration_b32: 8,
                onset_b32: 8,
            },
            Note {
                pitch: Pitch::C,
                octave: 4,
                duration_b32: 8,
                onset_b32: 16,
            },
            Note {
                pitch: Pitch::D,
                octave: 4,
                duration_b32: 8,
                onset_b32: 24,
            },
            Note {
                pitch: Pitch::E,
                octave: 4,
                duration_b32: 8,
                onset_b32: 32,
            },
            Note {
                pitch: Pitch::E,
                octave: 4,
                duration_b32: 8,
                onset_b32: 40,
            },
            Note {
                pitch: Pitch::E,
                octave: 4,
                duration_b32: 8,
                onset_b32: 48,
            },
            Note {
                pitch: Pitch::D,
                octave: 4,
                duration_b32: 8,
                onset_b32: 64,
            },
            Note {
                pitch: Pitch::D,
                octave: 4,
                duration_b32: 8,
                onset_b32: 72,
            },
            Note {
                pitch: Pitch::D,
                octave: 4,
                duration_b32: 8,
                onset_b32: 80,
            },
            Note {
                pitch: Pitch::E,
                octave: 4,
                duration_b32: 8,
                onset_b32: 96,
            },
            Note {
                pitch: Pitch::G,
                octave: 4,
                duration_b32: 8,
                onset_b32: 104,
            },
            Note {
                pitch: Pitch::G,
                octave: 4,
                duration_b32: 8,
                onset_b32: 112,
            },
            Note {
                pitch: Pitch::E,
                octave: 4,
                duration_b32: 8,
                onset_b32: 128,
            },
            Note {
                pitch: Pitch::D,
                octave: 4,
                duration_b32: 8,
                onset_b32: 136,
            },
            Note {
                pitch: Pitch::C,
                octave: 4,
                duration_b32: 8,
                onset_b32: 144,
            },
            Note {
                pitch: Pitch::D,
                octave: 4,
                duration_b32: 8,
                onset_b32: 152,
            },
            Note {
                pitch: Pitch::E,
                octave: 4,
                duration_b32: 8,
                onset_b32: 160,
            },
            Note {
                pitch: Pitch::E,
                octave: 4,
                duration_b32: 8,
                onset_b32: 168,
            },
            Note {
                pitch: Pitch::E,
                octave: 4,
                duration_b32: 8,
                onset_b32: 176,
            },
            Note {
                pitch: Pitch::E,
                octave: 4,
                duration_b32: 8,
                onset_b32: 184,
            },
            Note {
                pitch: Pitch::D,
                octave: 4,
                duration_b32: 8,
                onset_b32: 192,
            },
            Note {
                pitch: Pitch::D,
                octave: 4,
                duration_b32: 8,
                onset_b32: 200,
            },
            Note {
                pitch: Pitch::E,
                octave: 4,
                duration_b32: 8,
                onset_b32: 208,
            },
            Note {
                pitch: Pitch::D,
                octave: 4,
                duration_b32: 8,
                onset_b32: 216,
            },
            Note {
                pitch: Pitch::C,
                octave: 4,
                duration_b32: 8,
                onset_b32: 224,
            },
        ],
    }
}
