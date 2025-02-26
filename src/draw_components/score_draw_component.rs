use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};

use super::{DrawComponent, DrawResult, Position, ScoreViewDrawComponent, ViewportDrawResult};
use crate::cursor::Cursor;
use crate::events::InputEvent;
use crate::instruments::{self, InstrumentReference};
use crate::loop_state::LoopState;
use crate::pitch::Pitch;
use crate::player::PlayState;
use crate::score::{ActiveNote, NoteState, Score};
use crate::score_viewport::ScoreViewport;
use crate::selection_buffer::SelectionBuffer;
use log::debug;

pub struct ScoreDrawComponent {
    base: ScoreViewDrawComponent,
    instrument_references: Vec<InstrumentReference>,
}

impl DrawComponent for ScoreDrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult> {
        debug!(
            "Drawing score at position: x={}, y={}, w={}, h={}",
            pos.x, pos.y, pos.w, pos.h
        );

        self.draw_instruments(buffer, pos);

        let viewport_draw_result = self.draw_score(
            buffer,
            &Position {
                x: pos.x + 10,
                y: pos.y,
                w: pos.w - 10,
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
        instrument_references: Vec<InstrumentReference>,
    ) -> ScoreDrawComponent {
        ScoreDrawComponent {
            base: ScoreViewDrawComponent::new(
                score,
                play_state,
                score_viewport,
                tx,
                cursor,
                selection_buffer,
                loop_state,
            ),
            instrument_references,
        }
    }

    // Draw the instrument
    pub fn draw_instruments(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) {
        for (i, instrument) in self.instrument_references.iter().enumerate() {
            self.wb_string(buffer, pos, 0, i, instrument.name.clone())
        }
    }

    pub fn draw_grid(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) {
        for col in 0..pos.w - 1 {
            let bar_col = col % (self.base.score_viewport.resolution.bar_length_in_beats()) == 0;
            for (row, _instrument) in self.instrument_references.iter().enumerate() {
                let draw_char = if bar_col { '⎸' } else { '.' };
                self.wb(buffer, pos, col, row, draw_char);
            }

            if bar_col {
                let time_point_at_col = self.base.score_viewport.time_point
                    + (col as u64) * self.base.score_viewport.resolution.duration_b32();
                self.wb_string(
                    buffer,
                    pos,
                    col,
                    self.instrument_references.len(),
                    (time_point_at_col / (32)).to_string(),
                );
            }
        }
    }

    fn draw_score(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> ViewportDrawResult {
        debug!(
            "Drawing score with {} visible instruments",
            self.instrument_references.len()
        );

        // // Draw the empty grid with time markers
        self.draw_grid(buffer, pos);

        // // Draw playhead and loop markers
        let time_point = self
            .base
            .draw_playhead(buffer, pos, self.instrument_references.len());

        // // Draw notes with instrument-specific characters
        self.draw_score_notes(buffer, pos);

        // // Draw the cursor
        self.draw_cursor(buffer, pos);

        // ViewportDrawResult {
        //     pitch_low: *pitches.last().unwrap(),
        //     pitch_high: *pitches.first().unwrap(),
        //     time_point_start: self.base.score_viewport.time_point,
        //     time_point_end: time_point,
        // }
        ViewportDrawResult {
            pitch_low: Pitch::new(crate::pitch::Tone::C, 1),
            pitch_high: Pitch::new(crate::pitch::Tone::Cs, 1),
            time_point_start: 0,
            time_point_end: 200,
        }
    }

    pub fn draw_cursor(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) {
        // let mut time_point = self.base.score_viewport.time_point;
        // for col in 0..pos.w - 1 {
        //     for (row, _) in self.instrument_references.iter().enumerate() {
        //         // if self.cursor.visible_at(*pitch, time_point) {
        //         if true {
        //             self.wb(buffer, pos, col, row, 'C');
        //         }
        //     }
        //     time_point += self.base.score_viewport.resolution.duration_b32();
        // }
    }
    fn draw_score_notes(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) {
        let mut time_point = self.base.score_viewport.time_point;

        for col in 0..pos.w - 1 {
            // let mut col_states: HashMap<(usize, u32), (NoteState, u32)> = HashMap::new();
            let mut col_states: HashMap<(usize, u32), NoteState> = HashMap::new();

            for _ in 0..self.base.score_viewport.resolution.duration_b32() {
                let active_notes = self
                    .base
                    .score
                    .lock()
                    .unwrap()
                    .notes_active_at_time(time_point, None);

                for (row, instrument_ref) in self.instrument_references.iter().enumerate() {
                    if let Some(active_notes) = active_notes
                        .iter()
                        .filter(|note| note.note.instrument_id == instrument_ref.id)
                        .collect::<Vec<_>>()
                        .first()
                    {
                        let active_note = *active_notes;

                        col_states
                            .entry((row, instrument_ref.id))
                            .or_insert(NoteState::Sustain);

                        match active_note.state {
                            NoteState::Onset | NoteState::Release => {
                                col_states
                                    .entry((row, instrument_ref.id))
                                    .insert_entry(active_note.state);
                            }
                            NoteState::Sustain => (),
                        }
                    }
                }

                // if let SelectionBuffer::Score(ref selection_buffer_score) =
                //     self.base.selection_buffer
                // {
                //     let selected_notes =
                //         selection_buffer_score.notes_active_at_time(time_point, None);
                //     let selected_notes_map: HashMap<Pitch, ActiveNote> = selected_notes
                //         .into_iter()
                //         .map(|active_note| (active_note.note.pitch, active_note))
                //         .collect();

                //     for (row, pitch) in pitches.iter().enumerate() {
                //         if let Some(active_note) = selected_notes_map.get(pitch) {
                //             let current_state = col_states
                //                 .entry((row, *pitch))
                //                 .or_insert((NoteState::Sustain, active_note.note.instrument_id));
                //             current_state.0 = active_note.state;
                //             current_state.1 = active_note.note.instrument_id;
                //         }
                //     }
                // }

                time_point += 1;
            }

            // Render the notes with instrument-specific characters
            for ((row, _pitch), state) in col_states {
                let onset_char = '█';
                let sustain_char = '░';
                let release_char = '▒';
                let note_char = match state {
                    NoteState::Onset => onset_char,
                    NoteState::Sustain => sustain_char,
                    NoteState::Release => release_char,
                };

                DrawComponent::wb(self, buffer, pos, col, row, note_char);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loop_state::LoopMode;
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
            ScoreViewport::new(Pitch::new(Tone::C, 4), Resolution::Time1_16, 0, 0),
            tx,
            Cursor::new(Pitch::new(Tone::C, 4), 0),
            SelectionBuffer::None,
            LoopState::new(),
        )
    }

    fn create_buffer(width: usize, height: usize) -> Vec<Vec<char>> {
        vec![vec![' '; width]; height + 1]
    }

    #[test]
    fn test_visible_pitches() {
        let component = create_test_score_draw_component();
        let pos = Position {
            x: 0,
            y: 0,
            w: 10,
            h: 5,
        };

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
        let pos = Position {
            x: 0,
            y: 0,
            w: 10,
            h: 5,
        };

        let results = component.draw(&mut buffer, &pos);

        assert_eq!(results.len(), 1);
        if let DrawResult::ViewportDrawResult(vdr) = &results[0] {
            assert!(vdr.time_point_start <= vdr.time_point_end);
            assert!(vdr.pitch_low <= vdr.pitch_high);
        }
    }

    #[test]
    fn test_draw_notes() {
        let component = create_test_score_draw_component();
        let mut buffer = create_buffer(10, 5);
        let pos = Position {
            x: 0,
            y: 0,
            w: 10,
            h: 5,
        };

        {
            let mut score = component.base.score.lock().unwrap();
            score.insert(Pitch::new(Tone::C, 4), 0, 32, 0);
        }

        component.draw(&mut buffer, &pos);

        assert!(buffer[2].iter().any(|&c| c == '█' || c == '░' || c == '▒'));
    }

    #[test]
    fn test_draw_multiple_instruments() {
        let component = create_test_score_draw_component();
        let mut buffer = create_buffer(20, 7);
        let pos = Position {
            x: 0,
            y: 0,
            w: 20,
            h: 7,
        };

        {
            let mut score = component.base.score.lock().unwrap();
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
        let c4_row = buffer
            .iter()
            .position(|row| row.iter().take(4).collect::<String>().contains("C4 "))
            .expect("C4 row not found");

        let d4_row = buffer
            .iter()
            .position(|row| row.iter().take(4).collect::<String>().contains("D4 "))
            .expect("D4 row not found");

        // Check specific rows for instrument characters
        let has_instrument0_chars = buffer[c4_row]
            .iter()
            .any(|&c| c == '█' || c == '░' || c == '▒');
        let has_instrument1_chars = buffer[d4_row]
            .iter()
            .any(|&c| c == '◆' || c == '◇' || c == '◈');

        assert!(
            has_instrument0_chars,
            "Missing instrument 0 characters in C4 row"
        );
        assert!(
            has_instrument1_chars,
            "Missing instrument 1 characters in D4 row"
        );

        // Print detailed character counts per row
        for (i, row) in buffer.iter().enumerate() {
            let inst0_count = row
                .iter()
                .filter(|&&c| c == '█' || c == '░' || c == '▒')
                .count();
            let inst1_count = row
                .iter()
                .filter(|&&c| c == '◆' || c == '◇' || c == '◈')
                .count();
            if inst0_count > 0 || inst1_count > 0 {
                println!(
                    "Row {}: Instrument 0: {}, Instrument 1: {}",
                    i, inst0_count, inst1_count
                );
            }
        }
    }

    #[test]
    fn test_draw_cursor() {
        let mut component = create_test_score_draw_component();
        let mut buffer = create_buffer(10, 5);
        let pos = Position {
            x: 0,
            y: 0,
            w: 10,
            h: 5,
        };

        component.base.cursor = component.base.cursor.show();

        component.draw(&mut buffer, &pos);

        let has_cursor = buffer.iter().any(|row| row.iter().any(|&c| c == 'C'));
        assert!(has_cursor);
    }

    #[test]
    fn test_draw_loop_markers() {
        let mut component = create_test_score_draw_component();
        let mut buffer = create_buffer(10, 5);
        let pos = Position {
            x: 0,
            y: 0,
            w: 10,
            h: 5,
        };

        component.base.loop_state = component
            .base
            .loop_state
            .mark(0)
            .mark(32)
            .set_mode(LoopMode::Looping);

        component.draw(&mut buffer, &pos);

        let has_markers = buffer.iter().any(|row| row.iter().any(|&c| c == '░'));
        assert!(has_markers);
    }
}
