// score.rs

use std::{collections::HashMap, f32::MAX};

use crate::pitch::Pitch;

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub pitch: Pitch,
    pub onset_b32: u64,
    pub duration_b32: u64,
}

#[derive(Debug, Clone)]
pub struct Score {
    pub bpm: u16,
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

    pub fn time_within_song(&self, time_point_b32: u64) -> bool {
        let mut last_time_point_in_song = 0;

        for (_, notes_at_onset) in &self.notes {
            for note in notes_at_onset {
                if note.onset_b32 + note.duration_b32 > last_time_point_in_song {
                    last_time_point_in_song = note.onset_b32 + note.duration_b32
                }
            }
        }
        last_time_point_in_song > time_point_b32
    }

    pub fn insert_or_remove(&mut self, pitch: Pitch, onset_b32: u64, duration_b32: u64) {
        let mut notes_starting_at_time = self.notes_starting_at_time(onset_b32);

        let mut note_found_at_index = None;
        for (index, note) in notes_starting_at_time.iter().enumerate() {
            if note.pitch == pitch {
                note_found_at_index = Some(index);
            }
        }
        if let Some(matching_note_index) = note_found_at_index {
            notes_starting_at_time.remove(matching_note_index);
            self.notes.insert(onset_b32, notes_starting_at_time);
            return;
        }

        let note_to_insert = Note {
            pitch,
            onset_b32,
            duration_b32,
        };

        match self.notes.get_mut(&onset_b32) {
            Some(notes_at_onset) => {
                notes_at_onset.push(note_to_insert);
            }
            None => {
                self.notes.insert(onset_b32, vec![note_to_insert]);
            }
        }
    }

    // Creates a new Score with just notes between selection times and pitches.
    pub fn clone_at_selection(
        &self,
        time_point_start_b32: u64, // Inclusive
        time_point_end_b32: u64,   // Exclusive
        pitch_low: Pitch,
        pitch_high: Pitch,
    ) -> Score {
        let mut new_score = Score {
            bpm: self.bpm,
            notes: HashMap::new(),
        };

        for (&onset_b32, notes_at_onset) in &self.notes {
            if onset_b32 >= time_point_start_b32 && onset_b32 < time_point_end_b32 {
                for note in notes_at_onset {
                    if note.pitch >= pitch_low && note.pitch <= pitch_high {
                        // Assuming Pitch implements PartialOrd
                        new_score.insert_or_remove(note.pitch, note.onset_b32, note.duration_b32);
                    }
                }
            }
        }

        new_score
    }
}
