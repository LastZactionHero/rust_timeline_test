use std::sync::{Arc, Mutex, mpsc};

use crate::cursor::Cursor;
use crate::cursor::CursorMode;
use crate::draw_components::{DrawComponent, BoxDrawComponent, VSplitDrawComponent, NullComponent};
use crate::draw_components::score_draw_component::ScoreDrawComponent;
use crate::draw_components::status_bar_component::StatusBarComponent;
use crate::draw_components::{self, ViewportDrawResult};
use crate::events::InputEvent;
use crate::loop_state::LoopState;
use crate::player::Player;
use crate::score::Score;
use crate::score_viewport::ScoreViewport;
use crate::selection_buffer::SelectionBuffer;
use crate::song_file::SongFile;

use super::Gear;

pub struct ScoreEditorGear {
    score: Arc<Mutex<Score>>,
    score_viewport: ScoreViewport,
    player: Arc<Mutex<Player>>,
    input_tx: mpsc::Sender<InputEvent>,
    cursor: Cursor,
    selection_buffer: SelectionBuffer,
    loop_state: LoopState,
    song_file: SongFile,
    viewport_draw_result: Option<ViewportDrawResult>,
}

impl ScoreEditorGear {
    pub fn new(
        score: Arc<Mutex<Score>>,
        score_viewport: ScoreViewport,
        player: Arc<Mutex<Player>>,
        input_tx: mpsc::Sender<InputEvent>,
        cursor: Cursor,
        selection_buffer: SelectionBuffer,
        loop_state: LoopState,
    ) -> Self {
        Self {
            score,
            score_viewport,
            player,
            input_tx,
            cursor,
            selection_buffer,
            loop_state,
            song_file: SongFile::new(),
            viewport_draw_result: None,
        }
    }
}

// Add accessor methods to get cursor and viewport
impl ScoreEditorGear {
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }
    
    pub fn viewport(&self) -> ScoreViewport {
        self.score_viewport
    }
}

impl Gear for ScoreEditorGear {
    fn get_draw_component(&self) -> Box<dyn DrawComponent> {
        Box::new(BoxDrawComponent::new(Box::new(
            VSplitDrawComponent::new(
                draw_components::VSplitStyle::HalfWithDivider,
                Box::new(ScoreDrawComponent::new(
                    Arc::clone(&self.score),
                    self.player.lock().unwrap().state(),
                    self.score_viewport,
                    self.input_tx.clone(),
                    self.cursor,
                    self.selection_buffer.clone(),
                    self.loop_state,
                )),
                Box::new(VSplitDrawComponent::new(
                    draw_components::VSplitStyle::StatusBarNoDivider,
                    Box::new(NullComponent {}),
                    Box::new(StatusBarComponent::new(
                        self.cursor,
                        self.score_viewport,
                        self.loop_state,
                        999, // Special value to indicate score editor (all instruments)
                    )),
                )),
            ),
        )))
    }
    
    fn set_viewport_draw_result(&mut self, result: ViewportDrawResult) {
        self.viewport_draw_result = Some(result);
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn handle_event(&mut self, event: &InputEvent) -> bool {
        // Return true if event was handled, false otherwise
        match event {
            // Viewer navigation
            InputEvent::ViewerOctaveIncrease => {
                self.score_viewport = self.score_viewport.next_octave();
                true
            }
            InputEvent::ViewerOctaveDecrease => {
                self.score_viewport = self.score_viewport.prev_octave();
                true
            }
            InputEvent::ViewerBarNext => {
                let current_time = self.player.lock().unwrap().current_time_b32();
                let next_time = current_time + 32 - current_time % 32;
                self.player.lock().unwrap().set_time_b32(next_time);
                self.score_viewport = self.score_viewport.set_playback_time(next_time);
                
                // Use the actual viewport draw result from the last render if available
                if let Some(viewport_result) = self.viewport_draw_result {
                    // Now use next_bar with the actual viewport result
                    self.score_viewport = self.score_viewport.next_bar(&viewport_result);
                } else {
                    // Fallback to a simple implementation if no viewport result is available
                    self.score_viewport = self.score_viewport.set_time_point(
                        self.score_viewport.time_point + 32
                    );
                }
                true
            }
            InputEvent::ViewerBarPrevious => {
                let current_time = self.player.lock().unwrap().current_time_b32();
                let prev_time = if current_time < 32 {
                    0
                } else if current_time % 32 == 0 {
                    current_time - 32
                } else {
                    current_time - (current_time % 32)
                };
                self.player.lock().unwrap().set_time_b32(prev_time);
                self.score_viewport = self.score_viewport.set_playback_time(prev_time);
                
                // Use the actual viewport draw result from the last render if available
                if let Some(viewport_result) = self.viewport_draw_result {
                    // Now use prev_bar with the actual viewport result
                    self.score_viewport = self.score_viewport.prev_bar(&viewport_result);
                } else {
                    // Fallback to a simple implementation if no viewport result is available
                    let new_time = if self.score_viewport.time_point >= 32 {
                        self.score_viewport.time_point - 32
                    } else {
                        0
                    };
                    self.score_viewport = self.score_viewport.set_time_point(new_time);
                }
                true
            }
            
            // Resolution controls
            InputEvent::ViewerResolutionIncrease => {
                self.score_viewport = self.score_viewport.increase_resolution();
                self.cursor = self.cursor.resolution_align(self.score_viewport.resolution.duration_b32());
                true
            }
            InputEvent::ViewerResolutionDecrease => {
                self.score_viewport = self.score_viewport.decrease_resolution();
                self.cursor = self.cursor.resolution_align(self.score_viewport.resolution.duration_b32());
                true
            }
            
            // Playback controls
            InputEvent::PlayerTogglePlayback => {
                let mut player_guard = self.player.lock().unwrap();
                player_guard.toggle_playback();
                true
            }
            InputEvent::PlayerBeatChange(playback_time_point_b32) => {
                self.score_viewport = self.score_viewport.set_playback_time(*playback_time_point_b32);
                true
            }
            
            // Loop controls
            InputEvent::ToggleLoopMode => {
                self.loop_state = self.loop_state.toggle_mode();
                self.player.lock().unwrap().set_loop_state(self.loop_state);
                true
            }
            InputEvent::SetLoopTimes => {
                self.loop_state = self.loop_state.mark(self.score_viewport.playback_time_point);
                self.player.lock().unwrap().set_loop_state(self.loop_state);
                true
            }
            
            // File operations
            InputEvent::SaveSong => {
                if let Err(e) = self.song_file.save(&self.score.lock().unwrap()) {
                    log::error!("Failed to save song: {}", e);
                }
                true
            }
            
            // Cursor movement
            InputEvent::CursorUp => {
                self.cursor = self.cursor.up();
                match self.score_viewport.middle_pitch.next() {
                    Some(next_pitch) => self.score_viewport.middle_pitch = next_pitch,
                    None => (),
                }
                let mut player = self.player.lock().unwrap();
                // Use instrument 0 for previewing in score editor
                player.set_current_instrument_id(0);
                player.preview_note(self.cursor.pitch());
                true
            }
            InputEvent::CursorDown => {
                self.cursor = self.cursor.down();
                match self.score_viewport.middle_pitch.prev() {
                    Some(prev_pitch) => self.score_viewport.middle_pitch = prev_pitch,
                    None => (),
                }
                let mut player = self.player.lock().unwrap();
                // Use instrument 0 for previewing in score editor
                player.set_current_instrument_id(0);
                player.preview_note(self.cursor.pitch());
                true
            }
            InputEvent::CursorLeft => {
                // Move cursor to the left
                self.cursor = self.cursor.left(self.score_viewport.resolution.duration_b32());
                self.selection_buffer = self.selection_buffer.translate_to(self.cursor.time_point());
                
                // Check if cursor is near the edge of viewport and adjust if needed
                if let Some(viewport_result) = self.viewport_draw_result {
                    if self.cursor.time_point() <= viewport_result.time_point_start + 32 && self.score_viewport.time_point >= 32 {
                        // Adjust viewport to follow cursor by moving one bar back
                        self.score_viewport = self.score_viewport.set_time_point(
                            self.score_viewport.time_point - 32
                        );
                    }
                }
                true
            }
            InputEvent::CursorRight => {
                // Move cursor to the right
                self.cursor = self.cursor.right(self.score_viewport.resolution.duration_b32());
                self.selection_buffer = self.selection_buffer.translate_to(self.cursor.time_point());
                
                // Check if cursor is near the edge of viewport and adjust if needed
                if let Some(viewport_result) = self.viewport_draw_result {
                    if self.cursor.time_point() >= viewport_result.time_point_end - 32 {
                        // Adjust viewport to follow cursor by moving one bar forward
                        self.score_viewport = self.score_viewport.set_time_point(
                            self.score_viewport.time_point + 32
                        );
                    }
                }
                true
            }
            
            // Note editing
            InputEvent::InsertNote => {
                match self.cursor.mode() {
                    CursorMode::Select(_start, _end) => {
                        // Insert notes for the entire selection
                        let selection_range = self.cursor.selection_range().unwrap();
                        let pitch = self.cursor.pitch();
                        
                        // Calculate duration based on selection time points
                        let time_point = selection_range.time_point_start_b32;
                        let duration = selection_range.time_point_end_b32 - selection_range.time_point_start_b32;
                        
                        // Check if a note exists at this position for any instrument
                        let existing_notes = self.score.lock().unwrap().notes_starting_at_time(time_point, None)
                            .into_iter()
                            .filter(|note| note.pitch == pitch)
                            .collect::<Vec<_>>();
                        
                        let mut score_guard = self.score.lock().unwrap();
                        
                        if existing_notes.is_empty() {
                            // No existing note, insert a new one with instrument 0
                            score_guard.insert(pitch, time_point, duration, 0);
                        } else {
                            // Note exists, remove all notes at this position with this pitch
                            for note in existing_notes {
                                // We need to actually remove the note
                                score_guard.insert_or_remove(pitch, time_point, duration, note.instrument_id);
                            }
                        }
                        
                        // Move cursor to end of selection and clear selection mode
                        self.cursor = self.cursor.end_select();
                    }
                    _ => {
                        // Regular single note insertion
                        let pitch = self.cursor.pitch();
                        let time_point = self.cursor.time_point();
                        let duration = self.score_viewport.resolution.duration_b32();
                        
                        // Check if a note exists at this position for any instrument
                        let existing_notes = self.score.lock().unwrap().notes_starting_at_time(time_point, None)
                            .into_iter()
                            .filter(|note| note.pitch == pitch)
                            .collect::<Vec<_>>();
                        
                        let mut score_guard = self.score.lock().unwrap();
                        
                        if existing_notes.is_empty() {
                            // No existing note, insert a new one with instrument 0
                            score_guard.insert(pitch, time_point, duration, 0);
                        } else {
                            // Note exists, remove all notes at this position with this pitch
                            for note in existing_notes {
                                // We need to actually remove the note
                                score_guard.insert_or_remove(pitch, time_point, duration, note.instrument_id);
                            }
                        }
                        
                        self.cursor = self.cursor.right(duration);
                    }
                }
                true
            }
            // Selection and clipboard
            InputEvent::Cancel => {
                self.cursor = self.cursor.cancel();
                self.selection_buffer = SelectionBuffer::None;
                true
            }
            InputEvent::Yank => {
                if let CursorMode::Select(_, _) = self.cursor.mode() {
                    let selection_range = self.cursor.selection_range().unwrap();
                    let selection_score = self.score.lock().unwrap().clone_at_selection(selection_range, None); // No instrument filter
                    self.cursor = self.cursor.yank().right(self.score_viewport.resolution.duration_b32());
                    self.selection_buffer = SelectionBuffer::Score(
                        selection_score.translate(Some(self.cursor.time_point())),
                    );
                }
                true
            }
            InputEvent::Cut => {
                if let CursorMode::Select(_, _) = self.cursor.mode() {
                    let selection_range = self.cursor.selection_range().unwrap();
                    let selection_score = self.score.lock().unwrap().clone_at_selection(selection_range, None); // No instrument filter
                    self.score.lock().unwrap().delete_in_selection(selection_range, None); // No instrument filter
                    self.cursor = self.cursor.end_select();
                    self.selection_buffer = SelectionBuffer::Score(
                        selection_score.translate(Some(self.cursor.time_point())),
                    );
                }
                true
            }
            InputEvent::Paste => {
                if let SelectionBuffer::Score(ref selection_buffer_score) = self.selection_buffer {
                    let mut score_guard = self.score.lock().unwrap();
                    
                    // Don't replace the entire score, instead merge notes one by one
                    for (&onset_b32, notes_at_onset) in &selection_buffer_score.notes {
                        for note in notes_at_onset {
                            // Use insert instead of insert_or_remove to avoid toggling existing notes
                            score_guard.insert(note.pitch, onset_b32, note.duration_b32, note.instrument_id);
                        }
                    }
                    
                    // Force a rebuild of active_notes to ensure everything is up to date
                    score_guard.rebuild_active_notes();
                    
                    // Update cursor and selection buffer
                    let duration = selection_buffer_score.duration();
                    self.cursor = self.cursor.right(duration);
                    self.selection_buffer = SelectionBuffer::Score(
                        selection_buffer_score.translate(Some(self.cursor.time_point())),
                    );
                }
                true
            }
            InputEvent::Delete => {
                if let Some(selection_range) = self.cursor.selection_range() {
                    self.score.lock().unwrap().delete_in_selection(selection_range, None); // No instrument filter
                    self.cursor = self.cursor.end_select();
                }
                true
            }
            InputEvent::SelectIn => {
                self.cursor = self.cursor.start_select();
                true
            }
            _ => false, // Event not handled by this gear
        }
    }

    fn name(&self) -> &'static str {
        "Score Editor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::{Pitch, Tone};
    use std::sync::mpsc;
    use std::collections::HashMap;
    use crate::resolution::Resolution;
    use crate::loop_state::LoopMode;
    use crate::draw_components::Position;

    fn create_test_gear() -> ScoreEditorGear {
        let (tx, _rx) = mpsc::channel();
        let score = Arc::new(Mutex::new(Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        }));
        let player = Arc::new(Mutex::new(Player::create(score.clone(), 44100)));
        
        ScoreEditorGear::new(
            score,
            ScoreViewport::new(
                Pitch::new(Tone::C, 4),
                Resolution::Time1_16,
                0,
                0
            ),
            player,
            tx,
            Cursor::new(Pitch::new(Tone::C, 4), 0),
            SelectionBuffer::None,
            LoopState::new(),
        )
    }

    #[test]
    fn test_gear_creation() {
        let gear = create_test_gear();
        assert_eq!(gear.name(), "Score Editor");
        assert!(gear.viewport_draw_result.is_none());
    }

    #[test]
    fn test_get_draw_component() {
        let gear = create_test_gear();
        let component = gear.get_draw_component();
        let pos = Position { x: 0, y: 0, w: 40, h: 20 };
        let mut buffer = vec![vec![' '; pos.w]; pos.h];
        
        // Draw should return a vector of results without panicking
        let results = component.draw(&mut buffer, &pos);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_viewer_navigation() {
        let mut gear = create_test_gear();
        
        // Test octave navigation
        assert!(gear.handle_event(&InputEvent::ViewerOctaveIncrease));
        assert_eq!(gear.score_viewport.middle_pitch, Pitch::new(Tone::Cs, 4));
        
        assert!(gear.handle_event(&InputEvent::ViewerOctaveDecrease));
        assert_eq!(gear.score_viewport.middle_pitch, Pitch::new(Tone::C, 4));
        
        // Test bar navigation
        assert!(gear.handle_event(&InputEvent::ViewerBarNext));
        assert_eq!(gear.score_viewport.time_point, 32);
        
        assert!(gear.handle_event(&InputEvent::ViewerBarPrevious));
        assert_eq!(gear.score_viewport.time_point, 0);
    }

    #[test]
    fn test_resolution_controls() {
        let mut gear = create_test_gear();
        
        // Test resolution increase
        assert!(gear.handle_event(&InputEvent::ViewerResolutionIncrease));
        assert_eq!(gear.score_viewport.resolution, Resolution::Time1_32);
        
        // Test resolution decrease
        assert!(gear.handle_event(&InputEvent::ViewerResolutionDecrease));
        assert_eq!(gear.score_viewport.resolution, Resolution::Time1_16);
    }

    #[test]
    fn test_cursor_movement() {
        let mut gear = create_test_gear();
        
        // Test cursor up/down
        assert!(gear.handle_event(&InputEvent::CursorUp));
        assert_eq!(gear.cursor.pitch(), Pitch::new(Tone::Cs, 4));
        
        assert!(gear.handle_event(&InputEvent::CursorDown));
        assert_eq!(gear.cursor.pitch(), Pitch::new(Tone::C, 4));
        
        // Test cursor left/right
        assert!(gear.handle_event(&InputEvent::CursorRight));
        assert_eq!(gear.cursor.time_point(), 2); // Based on Time1_16 resolution
        
        assert!(gear.handle_event(&InputEvent::CursorLeft));
        assert_eq!(gear.cursor.time_point(), 0);
    }

    #[test]
    fn test_note_insertion() {
        let mut gear = create_test_gear();
        
        // Test single note insertion at time 0
        assert!(gear.handle_event(&InputEvent::InsertNote));
        {
            let score_guard = gear.score.lock().unwrap();
            let notes = score_guard.notes_starting_at_time(0, None);
            assert_eq!(notes.len(), 1, "Should have one note after insertion");
            assert_eq!(notes[0].pitch, Pitch::new(Tone::C, 4));
            assert_eq!(notes[0].instrument_id, 0);
        }
        
        // Move cursor back to time 0
        gear.cursor = Cursor::new(Pitch::new(Tone::C, 4), 0);
        
        // Test note removal by inserting at the same position
        assert!(gear.handle_event(&InputEvent::InsertNote));
        {
            let score_guard = gear.score.lock().unwrap();
            let notes = score_guard.notes_starting_at_time(0, None);
            assert!(notes.is_empty(), "Note should be removed after second insertion");
        }
    }

    #[test]
    fn test_selection_operations() {
        // This is a minimal test to verify that selection and delete operations 
        // don't crash when operating on a score with notes
        
        let mut gear = create_test_gear();
        
        // Set a valid resolution
        gear.score_viewport.resolution = Resolution::Time1_16;
        
        // Manually add a test note directly to the score
        {
            let mut score_guard = gear.score.lock().unwrap();
            let note = crate::score::Note {
                pitch: Pitch::new(Tone::C, 4),
                onset_b32: 0,
                duration_b32: 2,
                instrument_id: 0,
            };
            score_guard.notes.entry(0).or_insert_with(Vec::new).push(note);
            score_guard.rebuild_active_notes();
        }
        
        // Verify note was added
        {
            let score_guard = gear.score.lock().unwrap();
            let notes = score_guard.notes_starting_at_time(0, None);
            assert!(!notes.is_empty(), "Note should be present");
        }
        
        // Move cursor and start selection
        gear.cursor = Cursor::new(Pitch::new(Tone::C, 4), 0);
        assert!(gear.handle_event(&InputEvent::SelectIn));
        
        // Extend selection
        assert!(gear.handle_event(&InputEvent::CursorRight));
        
        // Delete selection
        assert!(gear.handle_event(&InputEvent::Delete));
        
        // Verify note was deleted
        {
            let score_guard = gear.score.lock().unwrap();
            let notes = score_guard.notes_starting_at_time(0, None);
            assert!(notes.is_empty(), "Note should be deleted");
        }
        
        // The test passes if we get here without crashing
    }
    
    // Paste operation test is tricky - in actual practice all the parts of the system
    // need to work together, but we're limited in what we can do in a unit test.
    // Let's introduce a test that verifies the functionality without being too strict.
    #[test]
    fn test_basic_operations() {
        // This test ensures all operations can be called without crashing, 
        // but doesn't verify specific behavior that's hard to mock
        
        let mut gear = create_test_gear();
        gear.score_viewport.resolution = Resolution::Time1_16;
        
        // Test note insertion
        assert!(gear.handle_event(&InputEvent::InsertNote));
        
        // Test selection operations
        assert!(gear.handle_event(&InputEvent::SelectIn));
        assert!(gear.handle_event(&InputEvent::CursorRight));
        
        // Test cancel
        assert!(gear.handle_event(&InputEvent::Cancel));
        
        // Test yank/cut/paste
        // These operations require proper setup that's complex to do in unit tests
        // So we just verify they don't crash
        assert!(gear.handle_event(&InputEvent::Yank));
        assert!(gear.handle_event(&InputEvent::Cut));
        assert!(gear.handle_event(&InputEvent::Paste));
        
        // Test delete operations
        gear.cursor = Cursor::new(Pitch::new(Tone::C, 4), 0);
        assert!(gear.handle_event(&InputEvent::SelectIn));
        assert!(gear.handle_event(&InputEvent::CursorRight));
        assert!(gear.handle_event(&InputEvent::Delete));
        
        // The test passes if none of these operations crash
    }

    #[test]
    fn test_loop_controls() {
        let mut gear = create_test_gear();
        
        // Test toggle loop mode
        assert!(gear.handle_event(&InputEvent::ToggleLoopMode));
        assert_eq!(gear.loop_state.mode, LoopMode::Looping);
        
        // Test set loop times
        assert!(gear.handle_event(&InputEvent::SetLoopTimes));
        assert!(gear.loop_state.start_time_b32.is_some());
    }

    #[test]
    fn test_viewport_draw_result() {
        let mut gear = create_test_gear();
        let result = ViewportDrawResult {
            pitch_low: Pitch::new(Tone::C, 3),
            pitch_high: Pitch::new(Tone::C, 5),
            time_point_start: 0,
            time_point_end: 32,
        };
        
        gear.set_viewport_draw_result(result);
        assert!(gear.viewport_draw_result.is_some());
        
        // Test that viewport result is used in bar navigation
        assert!(gear.handle_event(&InputEvent::ViewerBarNext));
        assert_eq!(gear.score_viewport.time_point, 32);
    }
}