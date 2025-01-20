// score.rs

use std::{collections::HashMap, f32::MAX};

use crate::pitch::Pitch;

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub pitch: Pitch,
    pub onset_b32: u64,
    pub duration_b32: u64,
}

pub struct Score {
    pub bpm: u16,
    // TODO: This needs to be a vector of notes
    pub notes: HashMap<u64, Vec<Note>>,
}

impl Score {
    pub fn notes_starting_at_time(&self, onset_b32: u64) -> Vec<Note> {
        self.notes
            .get(&onset_b32)
            .unwrap_or(&vec![])
            .iter()
            .map(|note| note.clone())
            .collect()
    }
    // pub fn note_state_at_time(
    //     &self,
    //     resolution: Resolution,
    //     time_b32: u64,
    //     pitch: Pitch,
    //     octave: u16,
    // ) -> NoteStateAtTime {
    //     let resolution_len = resolution.duration_b32();
    //     for note in &self.notes {
    //         if note.pitch != pitch
    //             || note.pitch.octave != octave
    //             || time_b32 >= note.onset_b32 + note.duration_b32
    //             || time_b32 + resolution_len - 1 < note.onset_b32
    //         {
    //             continue;
    //         }

    //         if time_b32 == note.onset_b32
    //             && time_b32 + resolution_len == note.onset_b32 + note.duration_b32
    //         {
    //             return NoteStateAtTime::Complete;
    //         } else if time_b32 <= note.onset_b32
    //             && time_b32 + resolution_len >= note.onset_b32 + note.duration_b32
    //         {
    //             return NoteStateAtTime::Enclosed;
    //         } else if time_b32 == note.onset_b32 {
    //             return NoteStateAtTime::Starting;
    //         } else if time_b32 > note.onset_b32
    //             && time_b32 < note.onset_b32 + note.duration_b32 - resolution_len
    //         {
    //             return NoteStateAtTime::Middle;
    //         }
    //         return NoteStateAtTime::Ending;
    //     }
    //     NoteStateAtTime::None
    // }

    // pub fn active_notes_at_time(&self, time_b32: u64) -> Vec<Note> {
    //     let mut active_notes = vec![];
    //     for note in &self.notes {
    //         if time_b32 >= note.onset_b32 && time_b32 < note.onset_b32 + note.duration_b32 {
    //             active_notes.push(*note);
    //         }
    //     }
    //     active_notes
    // }

    // pub fn time_within_song(&self, time_b32: u64) -> bool {
    //     let mut max_end_time = 0;
    //     for note in &self.notes {
    //         let end_time = note.onset_b32 + note.duration_b32;
    //         if end_time > max_end_time {
    //             max_end_time = end_time;
    //         }
    //     }
    //     time_b32 <= max_end_time
    // }
}
