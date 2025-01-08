// score.rs

use crate::pitch::Pitch;

#[derive(Debug)]
pub struct Note {
    pub pitch: Pitch,
    pub octave: u16,
    pub duration_b32: u64,
    pub onset_b32: u64,
}

pub struct Score {
    pub bpm: u16,
    pub notes: Vec<Note>,
}

#[derive(Clone, Copy)]
pub enum Resolution {
    Time1_4,
    Time1_8,
    Time1_16,
    Time1_32,
}

impl Resolution {
    pub fn as_str(&self) -> &str {
        match self {
            Resolution::Time1_4 => "1/4",
            Resolution::Time1_8 => "1/8",
            Resolution::Time1_16 => "1/16",
            Resolution::Time1_32 => "1/32",
        }
    }

    pub fn bar_length_in_beats(&self) -> u16 {
        match self {
            Resolution::Time1_4 => 4,
            Resolution::Time1_8 => 8,
            Resolution::Time1_16 => 16,
            Resolution::Time1_32 => 32,
        }
    }

    pub fn duration_b32(&self) -> u64 {
        match self {
            Resolution::Time1_4 => 8,
            Resolution::Time1_8 => 4,
            Resolution::Time1_16 => 2,
            Resolution::Time1_32 => 1,
        }
    }
}

pub enum NoteStateAtTime {
    None,
    Complete,
    Enclosed,
    Starting,
    Middle,
    Ending,
}

impl Score {
    pub fn note_state_at_time(
        &self,
        resolution: Resolution,
        time_b32: u64,
        pitch: Pitch,
        octave: u16,
    ) -> NoteStateAtTime {
        let resolution_len = resolution.duration_b32();
        for note in &self.notes {
            if note.pitch != pitch
                || note.octave != octave
                || time_b32 >= note.onset_b32 + note.duration_b32
                || time_b32 + resolution_len - 1 < note.onset_b32
            {
                continue;
            }

            if time_b32 == note.onset_b32
                && time_b32 + resolution_len == note.onset_b32 + note.duration_b32
            {
                return NoteStateAtTime::Complete;
            } else if time_b32 <= note.onset_b32
                && time_b32 + resolution_len >= note.onset_b32 + note.duration_b32
            {
                return NoteStateAtTime::Enclosed;
            } else if time_b32 == note.onset_b32 {
                return NoteStateAtTime::Starting;
            } else if time_b32 > note.onset_b32
                && time_b32 < note.onset_b32 + note.duration_b32 - resolution_len
            {
                return NoteStateAtTime::Middle;
            }
            return NoteStateAtTime::Ending;
        }
        NoteStateAtTime::None
    }
}
