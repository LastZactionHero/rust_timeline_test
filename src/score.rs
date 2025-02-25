// score.rs

use std::collections::HashMap;
use log::debug;

use crate::pitch::Pitch;
use crate::selection_range::SelectionRange;

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub pitch: Pitch,
    pub onset_b32: u64,
    pub duration_b32: u64,
    pub instrument_id: u32,  // Added instrument_id field with u32 type
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteState {
    Onset,
    Sustain,
    Release
}

#[derive(Debug, Clone)]
pub struct ActiveNote {
    pub note: Note,
    pub state: NoteState,
}

#[derive(Debug, Clone)]
pub struct Score {
    pub bpm: u16,
    pub notes: HashMap<u64, Vec<Note>>,
    pub active_notes: HashMap<u64, Vec<ActiveNote>>,
}

impl Score {
    /// Get notes starting at a specific time, optionally filtered by instrument ID
    pub fn notes_starting_at_time(&self, onset_b32: u64, instrument_id: Option<u32>) -> Vec<Note> {
        let empty_vec: Vec<Note> = Vec::new();
        let notes = self.notes.get(&onset_b32).unwrap_or(&empty_vec);
        
        match instrument_id {
            Some(id) => notes.iter()
                .filter(|note| note.instrument_id == id)
                .cloned()
                .collect(),
            None => notes.iter().cloned().collect()
        }
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

    pub fn insert_or_remove(&mut self, pitch: Pitch, onset_b32: u64, duration_b32: u64, instrument_id: u32) {
        let notes_starting_at_time = self.notes_starting_at_time(onset_b32, Some(instrument_id));

        let mut note_found_at_index = None;
        for (index, note) in notes_starting_at_time.iter().enumerate() {
            if note.pitch == pitch && note.instrument_id == instrument_id {
                note_found_at_index = Some(index);
            }
        }
        
        if let Some(matching_note_index) = note_found_at_index {
            let removed_note = notes_starting_at_time[matching_note_index];
            
            // Remove from active notes
            for t in removed_note.onset_b32..=removed_note.onset_b32 + removed_note.duration_b32 {
                if let Some(notes) = self.active_notes.get_mut(&t) {
                    notes.retain(|active| !(active.note.pitch == pitch && active.note.instrument_id == instrument_id));
                }
            }
            
            // Remove from notes starting at this time
            let mut all_notes_at_time = self.notes_starting_at_time(onset_b32, None);
            all_notes_at_time.retain(|note| !(note.pitch == pitch && note.instrument_id == instrument_id));
            
            if all_notes_at_time.is_empty() {
                self.notes.remove(&onset_b32);
            } else {
                self.notes.insert(onset_b32, all_notes_at_time);
            }
            return;
        }

        let note_to_insert = Note {
            pitch,
            onset_b32,
            duration_b32,
            instrument_id,
        };

        match self.notes.get_mut(&onset_b32) {
            Some(notes_at_onset) => {
                notes_at_onset.push(note_to_insert);
            }
            None => {
                self.notes.insert(onset_b32, vec![note_to_insert]);
            }
        }

        self.update_active_notes(note_to_insert);
    }

    /// Creates a new Score with just notes between selection times and pitches.
    /// Optionally filters by instrument_id
    pub fn clone_at_selection(&self, selection_range: SelectionRange, instrument_id: Option<u32>) -> Score {
        let mut new_score = Score {
            bpm: self.bpm,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

        for (&onset_b32, notes_at_onset) in &self.notes {
            if onset_b32 >= selection_range.time_point_start_b32 && onset_b32 < selection_range.time_point_end_b32 {
                for note in notes_at_onset {
                    // Check if note is within pitch range and (if specified) belongs to the right instrument
                    if note.pitch >= selection_range.pitch_low && note.pitch <= selection_range.pitch_high && 
                       (instrument_id.is_none() || instrument_id == Some(note.instrument_id)) {
                        // Assuming Pitch implements PartialOrd
                        new_score.insert_or_remove(note.pitch, note.onset_b32, note.duration_b32, note.instrument_id);
                    }
                }
            }
        }

        new_score
    }

    pub fn translate(&self, time_point_start_b32: Option<u64>) -> Score {
        match time_point_start_b32 {
            Some(new_start_time) => {
                let mut new_score = Score {
                    bpm: self.bpm,
                    notes: HashMap::new(),
                    active_notes: HashMap::new(),
                };

                let mut min_onset = u64::MAX;
                for (&onset_b32, _) in &self.notes {
                    min_onset = min_onset.min(onset_b32);
                }

                if min_onset == u64::MAX {
                    // No notes in the original score
                    return self.clone(); // Return a copy if no notes exist
                }

                let time_offset = if min_onset > new_start_time {
                    min_onset - new_start_time
                } else {
                    new_start_time - min_onset
                };

                for (&onset_b32, notes_at_onset) in &self.notes {
                    let new_onset = if min_onset > new_start_time {
                        onset_b32 - time_offset
                    } else {
                        onset_b32 + time_offset
                    };

                    for note in notes_at_onset {
                        new_score.insert_or_remove(note.pitch, new_onset, note.duration_b32, note.instrument_id);
                    }
                }

                new_score
            }
            None => self.clone(), // Return a copy if no new start time is provided
        }
    }

    pub fn insert(&mut self, pitch: Pitch, onset_b32: u64, duration_b32: u64, instrument_id: u32) {
        let end_b32 = onset_b32 + duration_b32;
        let mut overlapping_notes: Vec<(u64, Note)> = Vec::new();

        // Find all overlapping notes with the same pitch and instrument ID
        for (&existing_onset, notes) in &self.notes {
            for note in notes {
                if note.pitch == pitch && note.instrument_id == instrument_id {
                    let existing_end = note.onset_b32 + note.duration_b32;
                    // Check if notes strictly overlap (not just adjacent)
                    if !(existing_end <= onset_b32 || note.onset_b32 >= end_b32) {
                        overlapping_notes.push((existing_onset, *note));
                    }
                }
            }
        }

        // Remove all overlapping notes
        for (onset, note) in &overlapping_notes {
            if let Some(notes) = self.notes.get_mut(onset) {
                notes.retain(|n| !(n.pitch == note.pitch && n.instrument_id == note.instrument_id));
                if notes.is_empty() {
                    self.notes.remove(onset);
                }
            }
        }

        // Calculate merged note boundaries
        let merged_onset = if overlapping_notes.is_empty() {
            onset_b32
        } else {
            overlapping_notes
                .iter()
                .map(|(_, note)| note.onset_b32)
                .min()
                .unwrap()
                .min(onset_b32)
        };

        let merged_end = if overlapping_notes.is_empty() {
            end_b32
        } else {
            overlapping_notes
                .iter()
                .map(|(_, note)| note.onset_b32 + note.duration_b32)
                .max()
                .unwrap()
                .max(end_b32)
        };

        // Insert the merged note
        let merged_note = Note {
            pitch,
            onset_b32: merged_onset,
            duration_b32: merged_end - merged_onset,
            instrument_id,
        };

        match self.notes.get_mut(&merged_onset) {
            Some(notes_at_onset) => {
                notes_at_onset.push(merged_note);
            }
            None => {
                self.notes.insert(merged_onset, vec![merged_note]);
            }
        }

        self.update_active_notes(merged_note);
    }

    pub fn merge_down(&self, other: &Score) -> Score {
        let mut merged_score = self.clone();

        for (&onset_b32, notes_at_onset) in &other.notes {
            for note in notes_at_onset {
                // Use insert to properly handle overlapping notes
                merged_score.insert(note.pitch, onset_b32, note.duration_b32, note.instrument_id);
            }
        }

        // Make sure to rebuild active_notes after the merge
        merged_score.rebuild_active_notes();

        merged_score
    }
    
    /// Rebuilds the active_notes collection from the base notes
    pub fn rebuild_active_notes(&mut self) {
        // Clear and rebuild active_notes
        self.active_notes.clear();
        
        // Collect all notes from the score
        let all_notes: Vec<Note> = self.notes.values()
            .flat_map(|notes| notes.iter().cloned())
            .collect();
            
        // Rebuild active_notes
        for note in all_notes {
            self.update_active_notes(note);
        }
    }

    pub fn duration(&self) -> u64 {
        if self.notes.is_empty() {
            return 0; // Return 0 if the score is empty
        }

        let mut first_onset = u64::MAX;
        let mut last_final_time = 0;

        for (&onset_b32, notes_at_onset) in &self.notes {
            first_onset = first_onset.min(onset_b32);
            for note in notes_at_onset {
                last_final_time = last_final_time.max(note.onset_b32 + note.duration_b32);
            }
        }

        if first_onset == u64::MAX {
            // No notes found
            return 0;
        }

        last_final_time - first_onset
    }

    // Helper method to update active_notes when inserting/removing notes
    fn update_active_notes(&mut self, note: Note) {
        // Add new entries
        for t in note.onset_b32..=note.onset_b32 + note.duration_b32 - 1 {
            let state = if t == note.onset_b32 {
                NoteState::Onset
            } else if t == note.onset_b32 + note.duration_b32 - 1 {
                NoteState::Release
            } else {
                NoteState::Sustain
            };

            let active_note = ActiveNote {
                note,
                state,
            };
            
            if let Some(notes) = self.active_notes.get_mut(&t) {
                for note in notes {
                    if note.note.pitch == active_note.note.pitch {
                        if note.state == NoteState::Sustain {
                            continue;
                        }
                    }
                }
            }

            self.active_notes
                .entry(t)
                .or_insert_with(Vec::new)
                .push(active_note);
        }
    }

    /// Get active notes at a specific time, optionally filtered by instrument ID
    pub fn notes_active_at_time(&self, time_point_b32: u64, instrument_id: Option<u32>) -> Vec<ActiveNote> {
        let active_notes = self.active_notes
            .get(&time_point_b32)
            .cloned()
            .unwrap_or_default();
            
        match instrument_id {
            Some(id) => active_notes.into_iter()
                .filter(|active_note| active_note.note.instrument_id == id)
                .collect(),
            None => active_notes
        }
    }

    pub fn delete_in_selection(&mut self, selection_range: SelectionRange, instrument_id: Option<u32>) {
        debug!("Deleting notes between {} and {} with pitch range {:?} to {:?}, instrument_id: {:?}", 
            selection_range.time_point_start_b32, selection_range.time_point_end_b32, 
            selection_range.pitch_low, selection_range.pitch_high, instrument_id);

        let mut onsets_to_remove: Vec<u64> = Vec::new();
        let mut notes_to_keep: HashMap<u64, Vec<Note>> = HashMap::new();

        // Identify notes to remove and keep
        for (&onset_b32, notes_at_onset) in &self.notes {
            if onset_b32 >= selection_range.time_point_start_b32 && onset_b32 < selection_range.time_point_end_b32 {
                let (keep, remove): (Vec<Note>, Vec<Note>) = notes_at_onset
                    .iter()
                    .cloned()
                    .partition(|note| {
                        // Keep note if:
                        // 1. It's outside the pitch range OR
                        // 2. We're filtering by instrument_id and this note is for a different instrument
                        note.pitch < selection_range.pitch_low || 
                        note.pitch > selection_range.pitch_high || 
                        (instrument_id.is_some() && note.instrument_id != instrument_id.unwrap())
                    });

                debug!("At onset {}: keeping {} notes, removing {} notes", 
                    onset_b32, keep.len(), remove.len());

                if !keep.is_empty() {
                    notes_to_keep.insert(onset_b32, keep);
                } else {
                    onsets_to_remove.push(onset_b32);
                }
            }
        }

        debug!("Total onsets to remove: {}", onsets_to_remove.len());
        debug!("Total onsets with kept notes: {}", notes_to_keep.len());

        // Remove notes and update active_notes
        for onset_b32 in onsets_to_remove {
            self.notes.remove(&onset_b32);
        }

        // Update remaining onsets with kept notes
        for (onset_b32, notes) in notes_to_keep {
            self.notes.insert(onset_b32, notes);
        }

        // Rebuild active_notes
        self.rebuild_active_notes();
        
        debug!("Finished rebuilding active_notes");
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    fn create_test_score() -> Score {
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };
        // Add some test notes
        score.insert(Pitch::new(Tone::C, 4), 0, 32, 0); // C4 (MIDI 60), instrument 0
        score.insert(Pitch::new(Tone::E, 4), 32, 32, 0); // E4 (MIDI 64), instrument 0
        score.insert(Pitch::new(Tone::G, 4), 64, 32, 0); // G4 (MIDI 67), instrument 0
        score
    }

    #[test]
    fn test_notes_starting_at_time() {
        let score = create_test_score();

        // Test without filtering by instrument
        let notes = score.notes_starting_at_time(0, None);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].pitch, Pitch::new(Tone::C, 4));
        assert_eq!(notes[0].instrument_id, 0);

        // Test filtering by instrument 0
        let notes_inst0 = score.notes_starting_at_time(0, Some(0));
        assert_eq!(notes_inst0.len(), 1);
        assert_eq!(notes_inst0[0].instrument_id, 0);

        // Test filtering by non-existent instrument
        let notes_inst1 = score.notes_starting_at_time(0, Some(1));
        assert!(notes_inst1.is_empty());

        // Test time point with no notes
        let empty_notes = score.notes_starting_at_time(16, None);
        assert!(empty_notes.is_empty());
    }

    #[test]
    fn test_time_within_song() {
        let score = create_test_score();

        assert!(score.time_within_song(0));
        assert!(score.time_within_song(64));
        assert!(score.time_within_song(95));
        assert!(!score.time_within_song(96)); // Last note ends at 96
        assert!(!score.time_within_song(128));
    }

    #[test]
    fn test_insert_or_remove() {
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

        // Test insertion
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32, 0);
        assert_eq!(score.notes_starting_at_time(0, None).len(), 1);

        // Test removal
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32, 0);
        assert_eq!(score.notes_starting_at_time(0, None).len(), 0);
        
        // Test insertion with different instrument IDs
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32, 0);
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32, 1);
        
        // Should have 2 notes total
        assert_eq!(score.notes_starting_at_time(0, None).len(), 2);
        
        // Should have 1 note for instrument 0
        assert_eq!(score.notes_starting_at_time(0, Some(0)).len(), 1);
        
        // Should have 1 note for instrument 1
        assert_eq!(score.notes_starting_at_time(0, Some(1)).len(), 1);
        
        // Removing note for instrument 0 shouldn't affect instrument 1
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32, 0);
        assert_eq!(score.notes_starting_at_time(0, None).len(), 1);
        assert_eq!(score.notes_starting_at_time(0, Some(1)).len(), 1);
    }

    #[test]
    fn test_clone_at_selection() {
        let score = create_test_score();

        let selection_range = SelectionRange {
            time_point_start_b32: 0,
            time_point_end_b32: 64,
            pitch_low: Pitch::new(Tone::C, 4),
            pitch_high: Pitch::new(Tone::E, 4),
        };

        // Clone without filtering by instrument
        let selected = score.clone_at_selection(selection_range, None);

        assert_eq!(selected.notes_starting_at_time(0, None).len(), 1);
        assert_eq!(selected.notes_starting_at_time(32, None).len(), 1);
        assert_eq!(selected.notes_starting_at_time(64, None).len(), 0); // G4 is outside pitch range
        
        // Add note with a different instrument ID
        let mut score2 = score.clone();
        score2.insert_or_remove(Pitch::new(Tone::D, 4), 16, 32, 1);  // D4, instrument 1
        
        // Clone filtering by instrument 0
        let selected_inst0 = score2.clone_at_selection(selection_range, Some(0));
        assert_eq!(selected_inst0.notes_starting_at_time(0, None).len(), 1);   // C4 from instrument 0
        assert_eq!(selected_inst0.notes_starting_at_time(16, None).len(), 0);  // No D4 from instrument 1
        
        // Clone filtering by instrument 1
        let selected_inst1 = score2.clone_at_selection(selection_range, Some(1));
        assert_eq!(selected_inst1.notes_starting_at_time(0, None).len(), 0);   // No C4 from instrument 0
        assert_eq!(selected_inst1.notes_starting_at_time(16, None).len(), 1);  // D4 from instrument 1
    }

    #[test]
    fn test_translate() {
        let score = create_test_score();

        // Test translation to later time
        let translated = score.translate(Some(32));
        assert!(translated.notes_starting_at_time(0, None).is_empty());
        assert_eq!(
            translated.notes_starting_at_time(32, None)[0].pitch,
            Pitch::new(Tone::C, 4)
        );

        // Test translation with None
        let no_translation = score.translate(None);
        assert_eq!(no_translation.notes_starting_at_time(0, None).len(), 1);
        
        // Test translation preserves instrument IDs
        let mut multi_inst_score = Score { 
            bpm: 120, 
            notes: HashMap::new(), 
            active_notes: HashMap::new()
        };
        multi_inst_score.insert(Pitch::new(Tone::C, 4), 0, 32, 0);
        multi_inst_score.insert(Pitch::new(Tone::C, 4), 0, 32, 1);
        
        let translated_multi = multi_inst_score.translate(Some(32));
        assert_eq!(translated_multi.notes_starting_at_time(32, None).len(), 2);
        assert_eq!(translated_multi.notes_starting_at_time(32, Some(0)).len(), 1);
        assert_eq!(translated_multi.notes_starting_at_time(32, Some(1)).len(), 1);
    }

    #[test]
    fn test_insert() {
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

        // Test basic insertion
        score.insert(Pitch::new(Tone::C, 4), 0, 32, 0);
        assert_eq!(score.notes_starting_at_time(0, None).len(), 1);

        // Test overlapping notes merge within same instrument
        score.insert(Pitch::new(Tone::C, 4), 16, 32, 0);
        let notes = score.notes_starting_at_time(0, None);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].duration_b32, 48); // Notes should merge
        
        // Test notes with different instrument IDs don't merge
        score.insert(Pitch::new(Tone::C, 4), 0, 32, 1);
        let all_notes = score.notes_starting_at_time(0, None);
        assert_eq!(all_notes.len(), 2); // Should have 2 notes (one for each instrument)
        
        // Test merging only happens within instrument
        score.insert(Pitch::new(Tone::C, 4), 16, 32, 1);
        let inst1_notes = score.notes_starting_at_time(0, Some(1));
        assert_eq!(inst1_notes.len(), 1);
        assert_eq!(inst1_notes[0].duration_b32, 48); // Instrument 1 notes should merge
        
        // Instrument 0 note should remain unchanged
        let inst0_notes = score.notes_starting_at_time(0, Some(0));
        assert_eq!(inst0_notes.len(), 1);
        assert_eq!(inst0_notes[0].duration_b32, 48);
    }

    #[test]
    fn test_merge_down() {
        let mut score1 = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };
        score1.insert(Pitch::new(Tone::C, 4), 0, 32, 0);

        let mut score2 = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };
        score2.insert(Pitch::new(Tone::E, 4), 0, 32, 0);

        let merged = score1.merge_down(&score2);
        assert_eq!(merged.notes_starting_at_time(0, None).len(), 2);
        
        // Test merging across instruments
        let mut score3 = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };
        score3.insert(Pitch::new(Tone::G, 4), 0, 32, 1);  // Different instrument
        
        let merged2 = merged.merge_down(&score3);
        assert_eq!(merged2.notes_starting_at_time(0, None).len(), 3);
        assert_eq!(merged2.notes_starting_at_time(0, Some(0)).len(), 2);
        assert_eq!(merged2.notes_starting_at_time(0, Some(1)).len(), 1);
    }

    #[test]
    fn test_duration() {
        let empty_score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };
        assert_eq!(empty_score.duration(), 0);

        let score = create_test_score();
        assert_eq!(score.duration(), 96); // From start of first note to end of last note
    }

    #[test]
    fn test_note_states() {
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

        // Add a note from time 0 to 32
        score.insert(Pitch::new(Tone::C, 4), 0, 32, 0);

        // Test onset
        let notes_at_0 = score.notes_active_at_time(0, None);
        assert_eq!(notes_at_0.len(), 1);
        assert_eq!(notes_at_0[0].state, NoteState::Onset);
        assert_eq!(notes_at_0[0].note.pitch, Pitch::new(Tone::C, 4));
        assert_eq!(notes_at_0[0].note.instrument_id, 0);

        // Test sustain
        let notes_at_16 = score.notes_active_at_time(16, None);
        assert_eq!(notes_at_16.len(), 1);
        assert_eq!(notes_at_16[0].state, NoteState::Sustain);
        assert_eq!(notes_at_16[0].note.pitch, Pitch::new(Tone::C, 4));

        // Test release
        let notes_at_32 = score.notes_active_at_time(32, None);
        assert_eq!(notes_at_32.len(), 1);
        assert_eq!(notes_at_32[0].state, NoteState::Release);
        assert_eq!(notes_at_32[0].note.pitch, Pitch::new(Tone::C, 4));

        // Test no notes active
        let notes_at_33 = score.notes_active_at_time(33, None);
        assert_eq!(notes_at_33.len(), 0);
        
        // Add a note for a different instrument
        score.insert(Pitch::new(Tone::C, 4), 0, 32, 1);
        
        // Test filtering by instrument
        let notes_inst0 = score.notes_active_at_time(0, Some(0));
        assert_eq!(notes_inst0.len(), 1);
        assert_eq!(notes_inst0[0].note.instrument_id, 0);
        
        let notes_inst1 = score.notes_active_at_time(0, Some(1));
        assert_eq!(notes_inst1.len(), 1);
        assert_eq!(notes_inst1[0].note.instrument_id, 1);
    }

    #[test]
    fn test_overlapping_notes() {
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

        // Add two overlapping notes of the same pitch
        score.insert(Pitch::new(Tone::C, 4), 0, 32, 0);
        score.insert(Pitch::new(Tone::C, 4), 16, 32, 0);

        // Should be merged into one longer note
        let notes_at_0 = score.notes_active_at_time(0, None);
        assert_eq!(notes_at_0.len(), 1);
        assert_eq!(notes_at_0[0].state, NoteState::Onset);

        let notes_at_48 = score.notes_active_at_time(48, None);
        assert_eq!(notes_at_48.len(), 1);
        assert_eq!(notes_at_48[0].state, NoteState::Release);

        // Test that the note persists through the middle
        let notes_at_24 = score.notes_active_at_time(24, None);
        assert_eq!(notes_at_24.len(), 1);
        assert_eq!(notes_at_24[0].state, NoteState::Sustain);
        
        // Add overlapping notes for a different instrument
        score.insert(Pitch::new(Tone::C, 4), 0, 32, 1);
        score.insert(Pitch::new(Tone::C, 4), 16, 32, 1);
        
        // Now we should have two notes at time 0 (one for each instrument)
        let all_notes_at_0 = score.notes_active_at_time(0, None);
        assert_eq!(all_notes_at_0.len(), 2);
        
        // Instrument 0 notes should be merged
        let inst0_notes = score.notes_active_at_time(0, Some(0));
        assert_eq!(inst0_notes.len(), 1);
        
        // Instrument 1 notes should also be merged
        let inst1_notes = score.notes_active_at_time(0, Some(1));
        assert_eq!(inst1_notes.len(), 1);
    }

    #[test]
    fn test_remove_note() {
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

        // Add and then remove a note
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32, 0);
        
        // Verify note exists
        assert_eq!(score.notes_active_at_time(16, None).len(), 1);
        
        // Remove the note
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32, 0);
        
        // Verify note is gone from all time points
        assert_eq!(score.notes_active_at_time(0, None).len(), 0);
        assert_eq!(score.notes_active_at_time(16, None).len(), 0);
        assert_eq!(score.notes_active_at_time(32, None).len(), 0);
        
        // Test removing notes by instrument ID
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32, 0);
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32, 1);
        
        // Remove instrument 0 note but keep instrument 1 note
        score.insert_or_remove(Pitch::new(Tone::C, 4), 0, 32, 0);
        
        // Verify only instrument 0 note is gone
        assert_eq!(score.notes_active_at_time(0, None).len(), 1);
        assert_eq!(score.notes_active_at_time(0, Some(0)).len(), 0);
        assert_eq!(score.notes_active_at_time(0, Some(1)).len(), 1);
    }

    #[test]
    fn test_multiple_pitches() {
        let mut score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };

        // Add two notes at different pitches at the same time
        score.insert(Pitch::new(Tone::C, 4), 0, 32, 0);
        score.insert(Pitch::new(Tone::E, 4), 0, 32, 0);

        let notes_at_0 = score.notes_active_at_time(0, None);
        assert_eq!(notes_at_0.len(), 2);
        assert!(notes_at_0.iter().all(|n| n.state == NoteState::Onset));
        
        // Verify pitches are different
        let pitches: Vec<Pitch> = notes_at_0.iter().map(|n| n.note.pitch).collect();
        assert!(pitches.contains(&Pitch::new(Tone::C, 4)));
        assert!(pitches.contains(&Pitch::new(Tone::E, 4)));
        
        // Add same pitches for different instrument
        score.insert(Pitch::new(Tone::C, 4), 0, 32, 1);
        score.insert(Pitch::new(Tone::E, 4), 0, 32, 1);
        
        // Should now have 4 notes (2 for each instrument)
        assert_eq!(score.notes_active_at_time(0, None).len(), 4);
        assert_eq!(score.notes_active_at_time(0, Some(0)).len(), 2);
        assert_eq!(score.notes_active_at_time(0, Some(1)).len(), 2);
    }
}
