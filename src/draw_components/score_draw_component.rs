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

pub struct ScoreDrawComponent {
    score: Arc<Mutex<Score>>,
    play_state: PlayState,
    score_viewport: ScoreViewport,
    event_tx: mpsc::Sender<InputEvent>,
    cursor: Cursor,
    selection_buffer: SelectionBuffer,
    loop_state: LoopState,
}

impl DrawComponent for ScoreDrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) -> Vec<DrawResult> {
        debug!(
            "Drawing score at position: x={}, y={}, w={}, h={}",
            pos.x, pos.y, pos.w, pos.h
        );

        self.draw_pitches(buffer, pos);
        let viewport_draw_result = self.draw_score(
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

impl ScoreDrawComponent {
    pub fn new(
        score: Arc<Mutex<Score>>,
        play_state: PlayState,
        score_viewport: ScoreViewport,
        tx: mpsc::Sender<InputEvent>,
        cursor: Cursor,
        selection_buffer: SelectionBuffer,
        loop_state: LoopState,
    ) -> ScoreDrawComponent {
        ScoreDrawComponent {
            score,
            play_state,
            score_viewport,
            event_tx: tx,
            cursor,
            selection_buffer,
            loop_state,
        }
    }

    fn visible_pitches(&self, pos: &Position) -> Vec<Pitch> {
        let num_pitches_to_display = pos.h;

        let middle_pitch = self.score_viewport.middle_pitch;
        let mut pitches = vec![middle_pitch];
        for _ in 0..(num_pitches_to_display / 2) {
            if let Some(prev_pitch) = pitches.last().unwrap().prev() {
                pitches.push(prev_pitch);
            }
        }
        pitches.reverse();
        for _ in 0..(num_pitches_to_display / 2) {
            if let Some(next_pitch) = pitches.last().unwrap().next() {
                pitches.push(next_pitch);
            }
        }
        pitches.reverse();
        pitches
    }

    fn draw_score(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) -> ViewportDrawResult {
        let pitches = self.visible_pitches(pos);
        let _time_point = self.score_viewport.time_point;
        debug!("Drawing score with {} visible pitches", pitches.len());

        // Draw the empty score.
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
        let mut time_point = self.score_viewport.time_point;
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
            let mut col_states: HashMap<(usize, Pitch), (NoteState, u32)> = HashMap::new();

            for _ in 0..self.score_viewport.resolution.duration_b32() {
                let active_notes = self.score.lock().unwrap().notes_active_at_time(time_point, None);

                for (row, pitch) in pitches.iter().enumerate() {
                    if let Some(active_notes) = active_notes
                        .iter()
                        .filter(|note| note.note.pitch == *pitch)
                        .collect::<Vec<_>>()
                        .first()
                    {
                        let active_note = *active_notes;
                        let current_state = col_states
                            .entry((row, *pitch))
                            .or_insert((NoteState::Sustain, active_note.note.instrument_id));
                        
                        match active_note.state {
                            NoteState::Onset | NoteState::Release => {
                                current_state.0 = active_note.state;
                                current_state.1 = active_note.note.instrument_id;
                            }
                            NoteState::Sustain => {
                                if current_state.0 == NoteState::Sustain {
                                    current_state.0 = NoteState::Sustain;
                                    current_state.1 = active_note.note.instrument_id;
                                }
                            }
                        }
                    }
                }

                if let SelectionBuffer::Score(ref selection_buffer_score) = self.selection_buffer {
                    let selected_notes = selection_buffer_score.notes_active_at_time(time_point, None);
                    let selected_notes_map: HashMap<Pitch, ActiveNote> = selected_notes
                        .into_iter()
                        .map(|active_note| (active_note.note.pitch, active_note))
                        .collect();

                    for (row, pitch) in pitches.iter().enumerate() {
                        if let Some(active_note) = selected_notes_map.get(pitch) {
                            let current_state = col_states
                                .entry((row, *pitch))
                                .or_insert((NoteState::Sustain, active_note.note.instrument_id));
                            current_state.0 = active_note.state;
                            current_state.1 = active_note.note.instrument_id;
                        }
                    }
                }

                time_point += 1;
            }

            // Render the notes with instrument-specific characters
            for ((row, _pitch), (state, instrument_id)) in col_states {
                // Choose different characters for each instrument
                let instrument_chars = [
                    ('█', '░', '▒'), // Instrument 0 (solid blocks)
                    ('◆', '◇', '◈'), // Instrument 1 (diamonds)
                    ('●', '○', '◍'), // Instrument 2 (circles)
                    ('▲', '△', '▴'), // Instrument 3 (triangles)
                ];
                
                // Fallback to default characters if instrument ID is out of range
                let (onset_char, sustain_char, release_char) = 
                    if instrument_id < instrument_chars.len() as u32 {
                        instrument_chars[instrument_id as usize]
                    } else {
                        ('X', 'x', '+') // Default fallback
                    };
                
                let note_char = match state {
                    NoteState::Onset => onset_char,
                    NoteState::Sustain => sustain_char,
                    NoteState::Release => release_char,
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
            pitch_low: *pitches.last().unwrap(),
            pitch_high: *pitches.first().unwrap(),
            time_point_start: self.score_viewport.time_point,
            time_point_end: time_point,
        }
    }

    fn draw_pitches(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) {
        for (i, pitch) in self.visible_pitches(pos).iter().enumerate() {
            self.wb_string(buffer, pos, 0, i, pitch.as_str());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::{Pitch, Tone};
    use crate::resolution::Resolution;
    use std::sync::mpsc;

    fn create_test_score_draw_component() -> ScoreDrawComponent {
        let (tx, _rx) = mpsc::channel();
        let score = Arc::new(Mutex::new(Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        }));
        
        ScoreDrawComponent::new(
            score,
            PlayState::Stopped,
            ScoreViewport::new(
                Pitch::new(Tone::C, 4),
                Resolution::Time1_16,
                0,
                0
            ),
            tx,
            Cursor::new(Pitch::new(Tone::C, 4), 0),
            SelectionBuffer::None,
            LoopState::new()
        )
    }

    fn create_buffer(width: usize, height: usize) -> Vec<Vec<char>> {
        vec![vec![' '; width]; height + 1]
    }

    #[test]
    fn test_visible_pitches() {
        let component = create_test_score_draw_component();
        let pos = Position { x: 0, y: 0, w: 10, h: 5 };
        
        let pitches = component.visible_pitches(&pos);
        
        assert_eq!(pitches.len(), 5);
        assert_eq!(pitches[2], Pitch::new(Tone::C, 4)); // Middle pitch should be C4
        assert!(pitches[0] > pitches[1]); // Should be ordered high to low
        assert!(pitches[3] > pitches[4]);
    }

    #[test]
    fn test_draw_empty_score() {
        let component = create_test_score_draw_component();
        let mut buffer = create_buffer(10, 5);
        let pos = Position { x: 0, y: 0, w: 10, h: 5 };
        
        let results = component.draw(&mut buffer, &pos);
        
        assert_eq!(results.len(), 1);
        match &results[0] {
            DrawResult::ViewportDrawResult(vdr) => {
                assert!(vdr.time_point_start <= vdr.time_point_end);
                assert!(vdr.pitch_low <= vdr.pitch_high);
            },
            _ => panic!("Expected ViewportDrawResult"),
        }
    }

    #[test]
    fn test_draw_notes() {
        let component = create_test_score_draw_component();
        let mut buffer = create_buffer(10, 5);
        let pos = Position { x: 0, y: 0, w: 10, h: 5 };
        
        {
            let mut score = component.score.lock().unwrap();
            score.insert(Pitch::new(Tone::C, 4), 0, 32, 0);
        }
        
        component.draw(&mut buffer, &pos);
        
        assert!(buffer[2].iter().any(|&c| c == '█' || c == '░' || c == '▒'));
    }

    #[test]
    fn test_draw_multiple_instruments() {
        let component = create_test_score_draw_component();
        let mut buffer = create_buffer(20, 7);
        let pos = Position { x: 0, y: 0, w: 20, h: 7 };
        
        {
            let mut score = component.score.lock().unwrap();
            // Place notes at different pitches but same time
            score.insert(Pitch::new(Tone::C, 4), 0, 32, 0); // Instrument 0, lower pitch
            score.insert(Pitch::new(Tone::D, 4), 0, 32, 1); // Instrument 1, higher pitch
            
            // Add some notes at a different time point to ensure visibility
            score.insert(Pitch::new(Tone::E, 4), 32, 32, 0); // Instrument 0
            score.insert(Pitch::new(Tone::F, 4), 32, 32, 1); // Instrument 1
            
            // Force a rebuild of active notes to ensure everything is up to date
            score.rebuild_active_notes();
        }
        
        component.draw(&mut buffer, &pos);
        
        // Debug print the buffer to see what's being drawn
        println!("\nDebug buffer contents:");
        for row in &buffer {
            let row_str: String = row.iter().collect();
            println!("{}", row_str);
        }
        
        // Check for specific instrument characters in specific rows
        let c4_row = buffer.iter()
            .position(|row| row.iter().take(4).collect::<String>().contains("C4 "))
            .expect("C4 row not found");
        
        let d4_row = buffer.iter()
            .position(|row| row.iter().take(4).collect::<String>().contains("D4 "))
            .expect("D4 row not found");
        
        // Check specific rows for instrument characters
        let has_instrument0_chars = buffer[c4_row].iter()
            .any(|&c| c == '█' || c == '░' || c == '▒');
        let has_instrument1_chars = buffer[d4_row].iter()
            .any(|&c| c == '◆' || c == '◇' || c == '◈');
        
        assert!(has_instrument0_chars, "Missing instrument 0 characters in C4 row");
        assert!(has_instrument1_chars, "Missing instrument 1 characters in D4 row");
        
        // Print detailed character counts per row
        for (i, row) in buffer.iter().enumerate() {
            let inst0_count = row.iter()
                .filter(|&&c| c == '█' || c == '░' || c == '▒')
                .count();
            let inst1_count = row.iter()
                .filter(|&&c| c == '◆' || c == '◇' || c == '◈')
                .count();
            if inst0_count > 0 || inst1_count > 0 {
                println!("Row {}: Instrument 0: {}, Instrument 1: {}", i, inst0_count, inst1_count);
            }
        }
    }

    #[test]
    fn test_draw_cursor() {
        let mut component = create_test_score_draw_component();
        let mut buffer = create_buffer(10, 5);
        let pos = Position { x: 0, y: 0, w: 10, h: 5 };
        
        component.cursor = component.cursor.show();
        
        component.draw(&mut buffer, &pos);
        
        let has_cursor = buffer.iter().any(|row| row.iter().any(|&c| c == 'C'));
        assert!(has_cursor);
    }

    #[test]
    fn test_draw_loop_markers() {
        let mut component = create_test_score_draw_component();
        let mut buffer = create_buffer(10, 5);
        let pos = Position { x: 0, y: 0, w: 10, h: 5 };
        
        component.loop_state = component.loop_state
            .mark(0)
            .mark(32)
            .set_mode(LoopMode::Looping);
        
        component.draw(&mut buffer, &pos);
        
        let has_markers = buffer.iter().any(|row| row.iter().any(|&c| c == '░'));
        assert!(has_markers);
    }
}