use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};

use super::{DrawComponent, DrawResult, ViewportDrawResult};
use crate::cursor::Cursor;
use crate::draw_components::Position;
use crate::events::InputEvent;
use crate::pitch::Pitch;
use crate::player::PlayState;
use crate::score::{ActiveNote, NoteState, Score};
use crate::score_viewport::ScoreViewport;
use crate::selection_buffer::SelectionBuffer;
use log::debug;
use crate::loop_state::{LoopState, LoopMode};

pub struct TrackDrawComponent {
    score: Arc<Mutex<Score>>,
    play_state: PlayState,
    score_viewport: ScoreViewport,
    event_tx: mpsc::Sender<InputEvent>,
    cursor: Cursor,
    selection_buffer: SelectionBuffer,
    loop_state: LoopState,
    instrument_id: u32,  // Add instrument ID to the component
}

impl DrawComponent for TrackDrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) -> Vec<DrawResult> {
        debug!(
            "Drawing track at position: x={}, y={}, w={}, h={}",
            pos.x, pos.y, pos.w, pos.h
        );

        self.draw_pitches(buffer, pos);
        let viewport_draw_result = self.draw_track(
            buffer,
            &Position {
                x: pos.x + 4,
                y: pos.y,
                w: pos.w - 4,
                h: pos.h,
            },
        );
        vec![DrawResult::ViewportDrawResult(viewport_draw_result)]
    }
}

impl TrackDrawComponent {
    pub fn new(
        score: Arc<Mutex<Score>>,
        play_state: PlayState,
        score_viewport: ScoreViewport,
        event_tx: mpsc::Sender<InputEvent>,
        cursor: Cursor,
        selection_buffer: SelectionBuffer,
        loop_state: LoopState,
        instrument_id: u32,
    ) -> TrackDrawComponent {
        TrackDrawComponent {
            score,
            play_state,
            score_viewport,
            event_tx,
            cursor,
            selection_buffer,
            loop_state,
            instrument_id,
        }
    }
    
    // Helper method to write a character to the buffer
    fn wb(&self, buffer: &mut Vec<Vec<char>>, pos: &Position, x: usize, y: usize, c: char) {
        let x = pos.x + x;
        let y = pos.y + y;
        if x < buffer[0].len() && y < buffer.len() {
            buffer[y][x] = c;
        }
    }

    // Helper method to write a string to the buffer
    fn wb_string(&self, buffer: &mut Vec<Vec<char>>, pos: &Position, x: usize, y: usize, s: String) {
        for (i, c) in s.chars().enumerate() {
            self.wb(buffer, pos, x + i, y, c);
        }
    }

    fn visible_pitches(&self, pos: &Position) -> Vec<Pitch> {
        let num_pitches = pos.h - 1;  // Subtract 1 for bar numbers row
        let middle_index = num_pitches / 2;
        let middle_pitch = self.score_viewport.middle_pitch;
        
        let mut pitches = vec![middle_pitch];
        
        // Add higher pitches
        let mut current = middle_pitch;
        for _ in 0..middle_index {
            if let Some(next) = current.next() {
                pitches.insert(0, next);  // Insert at start to maintain high-to-low order
                current = next;
            }
        }
        
        // Add lower pitches
        current = middle_pitch;
        for _ in 0..(num_pitches - middle_index - 1) {
            if let Some(prev) = current.prev() {
                pitches.push(prev);  // Add at end for lower pitches
                current = prev;
            }
        }
        
        pitches
    }

    fn draw_track(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> ViewportDrawResult {
        let pitches = self.visible_pitches(pos);
        let mut time_point = self.score_viewport.time_point;
        debug!("Drawing track with {} visible pitches", pitches.len());

        // Draw the empty track.
        for col in 0..pos.w - 1 {
            let bar_col = col % (self.score_viewport.resolution.bar_length_in_beats()) == 0;
            for (row, _pitch) in pitches.iter().enumerate() {
                let draw_char = if bar_col { '⎸' } else { '.' };
                self.wb(buffer, pos, col, row, draw_char);
            }

            if bar_col {
                let time_point_at_col = self.score_viewport.time_point
                    + (col as u64) * self.score_viewport.resolution.duration_b32();
                self.wb_string(
                    buffer,
                    pos,
                    col,
                    pitches.len(),
                    (time_point_at_col / (32)).to_string(),
                );
            }
        }

        // Draw the playhead and loop markers
        for col in 0..pos.w - 1 {
            for _ in 0..self.score_viewport.resolution.duration_b32() {
                for (row, _pitch) in pitches.iter().enumerate() {
                    if time_point == self.score_viewport.playback_time_point {
                        self.wb(buffer, pos, col, row, '░');
                    } else if self.loop_state.mode == LoopMode::Looping {
                        // Show loop start/end markers if loop mode is enabled
                        if let Some(start_time) = self.loop_state.start_time_b32 {
                            if time_point == start_time {
                                self.wb(buffer, pos, col, row, '░');
                            }
                        }
                        if let Some(end_time) = self.loop_state.end_time_b32 {
                            if time_point == end_time {
                                self.wb(buffer, pos, col, row, '░');
                            }
                        }
                    }
                }
                time_point += 1;
            }
        }

        let mut time_point = self.score_viewport.time_point;
        for col in 0..pos.w - 1 {
            let mut col_states: HashMap<(usize, Pitch), NoteState> = HashMap::new();

            for _ in 0..self.score_viewport.resolution.duration_b32() {
                // Use instrument_id when querying for active notes
                let active_notes = self.score.lock().unwrap().notes_active_at_time(time_point, Some(self.instrument_id));

                for (row, pitch) in pitches.iter().enumerate() {
                    if let Some(active_note) =
                        active_notes.iter().find(|note| note.note.pitch == *pitch)
                    {
                        let current_state = col_states
                            .entry((row, *pitch))
                            .or_insert(NoteState::Sustain);
                        match active_note.state {
                            NoteState::Onset | NoteState::Release => {
                                *current_state = active_note.state
                            }
                            NoteState::Sustain => {
                                if *current_state == NoteState::Sustain {
                                    *current_state = NoteState::Sustain
                                }
                            }
                        }
                    }
                }

                if let SelectionBuffer::Score(ref selection_buffer_score) = self.selection_buffer {
                    // Filter selection buffer notes by instrument_id as well
                    let selected_notes = selection_buffer_score.notes_active_at_time(time_point, Some(self.instrument_id));
                    let selected_notes_map: HashMap<Pitch, ActiveNote> = selected_notes
                        .into_iter()
                        .map(|active_note| (active_note.note.pitch, active_note))
                        .collect();

                    for (row, pitch) in pitches.iter().enumerate() {
                        if let Some(active_note) = selected_notes_map.get(pitch) {
                            let current_state = col_states
                                .entry((row, *pitch))
                                .or_insert(NoteState::Sustain);
                            *current_state = active_note.state;
                            match active_note.state {
                                NoteState::Onset => *current_state = NoteState::Onset,
                                NoteState::Sustain => *current_state = NoteState::Sustain,
                                NoteState::Release => *current_state = NoteState::Release,
                            }
                        }
                    }
                }

                time_point += 1;
            }

            for ((row, _pitch), state) in col_states {
                let note_char = match state {
                    NoteState::Onset => '█',
                    NoteState::Sustain => '░',
                    NoteState::Release => '▒',
                };
                self.wb(buffer, pos, col, row, note_char);
            }
        }

        // Draw the cursor - iterate over each time point and pitch.
        let mut time_point = self.score_viewport.time_point;
        for col in 0..pos.w - 1 {
            for (row, pitch) in pitches.iter().enumerate() {
                if self.cursor.visible_at(*pitch, time_point) {
                    self.wb(buffer, pos, col, row, 'C');
                }
            }
            time_point += self.score_viewport.resolution.duration_b32();
        }

        ViewportDrawResult {
            pitch_high: pitches.first().cloned().unwrap_or(self.score_viewport.middle_pitch),
            pitch_low: pitches.last().cloned().unwrap_or(self.score_viewport.middle_pitch),
            time_point_start: self.score_viewport.time_point,
            time_point_end: time_point,
        }
    }

    fn draw_pitches(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) {
        let pitches = self.visible_pitches(pos);
        for (i, pitch) in pitches.iter().enumerate() {
            self.wb_string(buffer, pos, 0, i, pitch.as_str());  // Use as_str() instead of to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::Tone;
    use crate::resolution::Resolution;
    use crate::player::PlayState;

    fn create_test_track_component(instrument_id: u32) -> TrackDrawComponent {
        let score = Arc::new(Mutex::new(Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        }));
        
        let score_viewport = ScoreViewport::new(
            Pitch::new(Tone::C, 4),
            Resolution::Time1_16,
            32,
            0
        );
        
        let (tx, _rx) = mpsc::channel();
        
        TrackDrawComponent::new(
            score,
            PlayState::Stopped,
            score_viewport,
            tx,
            Cursor::new(Pitch::new(Tone::C, 4), 32),
            SelectionBuffer::None,
            LoopState::new(),
            instrument_id
        )
    }

    #[test]
    fn test_visible_pitches() {
        let component = create_test_track_component(0);
        let pos = Position { x: 0, y: 0, w: 10, h: 5 };
        
        let pitches = component.visible_pitches(&pos);
        
        // Should have h-1 pitches (one row reserved for bar numbers)
        assert_eq!(pitches.len(), 4);
        
        // Middle pitch should be C4 (viewport middle pitch)
        assert_eq!(pitches[pitches.len()/2], Pitch::new(Tone::C, 4));
    }

    #[test]
    fn test_draw_track_empty() {
        let component = create_test_track_component(0);
        let mut buffer = vec![vec![' '; 20]; 10];
        let pos = Position { x: 0, y: 0, w: 20, h: 5 };
        
        let result = component.draw_track(&mut buffer, &pos);
        
        // Verify viewport result
        assert!(result.time_point_start == 32);
        assert!(result.time_point_end > result.time_point_start);
        assert!(result.pitch_high > result.pitch_low);
    }

    #[test]
    fn test_draw_track_with_notes() {
        let component = create_test_track_component(0);
        
        // Add some test notes for instrument 0
        {
            let mut score = component.score.lock().unwrap();
            score.insert(Pitch::new(Tone::C, 4), 32, 32, 0);
            score.rebuild_active_notes();
        }
        
        let mut buffer = vec![vec![' '; 20]; 10];
        let pos = Position { x: 0, y: 0, w: 20, h: 5 };
        
        component.draw_track(&mut buffer, &pos);
        
        // Verify notes are drawn only for instrument 0
        let mut found_note = false;
        for row in buffer {
            for cell in row {
                if cell == '█' || cell == '░' || cell == '▒' {
                    found_note = true;
                }
            }
        }
        assert!(found_note, "Expected to find note characters in buffer");
    }

    #[test]
    fn test_draw_track_multiple_instruments() {
        let component = create_test_track_component(0);
        
        // Add notes for different instruments
        {
            let mut score = component.score.lock().unwrap();
            score.insert(Pitch::new(Tone::C, 4), 32, 32, 0); // Should be visible
            score.insert(Pitch::new(Tone::C, 4), 32, 32, 1); // Should be filtered out
            score.rebuild_active_notes();
        }
        
        let mut buffer = vec![vec![' '; 20]; 10];
        let pos = Position { x: 0, y: 0, w: 20, h: 5 };
        
        component.draw_track(&mut buffer, &pos);
        
        // Count note characters (should only find notes for instrument 0)
        let note_count = buffer.iter()
            .flat_map(|row| row.iter())
            .filter(|&&c| c == '█' || c == '░' || c == '▒')
            .count();
            
        assert!(note_count > 0, "Expected to find notes for instrument 0");
    }

    #[test]
    fn test_draw_track_with_selection_buffer() {
        let component = create_test_track_component(0);
        let mut selection_score = Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        };
        
        // Add a note to the selection buffer
        selection_score.insert(Pitch::new(Tone::C, 4), 32, 32, 0);
        
        let mut component_with_selection = component;
        component_with_selection.selection_buffer = SelectionBuffer::Score(selection_score);
        
        let mut buffer = vec![vec![' '; 20]; 10];
        let pos = Position { x: 0, y: 0, w: 20, h: 5 };
        
        component_with_selection.draw_track(&mut buffer, &pos);
        
        // Verify selection buffer notes are drawn
        let mut found_selection = false;
        for row in buffer {
            for cell in row {
                if cell == '█' || cell == '░' || cell == '▒' {
                    found_selection = true;
                }
            }
        }
        assert!(found_selection, "Expected to find selection buffer notes");
    }

    #[test]
    fn test_draw_pitches() {
        let component = create_test_track_component(0);
        let mut buffer = vec![vec![' '; 20]; 10];
        let pos = Position { x: 0, y: 0, w: 20, h: 5 };
        
        component.draw_pitches(&mut buffer, &pos);
        
        // Verify pitch labels are drawn
        assert!(buffer[2][0] == 'C', "Expected to find middle C pitch label");
    }
}