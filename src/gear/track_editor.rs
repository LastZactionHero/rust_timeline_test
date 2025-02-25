use std::sync::{Arc, Mutex, mpsc};

use crate::cursor::Cursor;
use crate::cursor::CursorMode;
use crate::draw_components::{DrawComponent, BoxDrawComponent, VSplitDrawComponent, NullComponent};
use crate::draw_components::track_draw_component::TrackDrawComponent;
use crate::draw_components::status_bar_component::StatusBarComponent;
use crate::draw_components::{self, ViewportDrawResult};
use crate::events::InputEvent;
use crate::loop_state::LoopState;
use crate::player::Player;
use crate::score::Score;
use crate::score_viewport::ScoreViewport;
use crate::selection_buffer::SelectionBuffer;
use crate::song_file::SongFile;
use log::error;

use super::Gear;

pub struct TrackEditorGear {
    score: Arc<Mutex<Score>>,
    score_viewport: ScoreViewport,
    player: Arc<Mutex<Player>>,
    input_tx: mpsc::Sender<InputEvent>,
    cursor: Cursor,
    selection_buffer: SelectionBuffer,
    loop_state: LoopState,
    song_file: SongFile,
    viewport_draw_result: Option<ViewportDrawResult>,
    instrument_id: u32,  // Instrument ID for this track editor
}

impl TrackEditorGear {
    pub fn new(
        score: Arc<Mutex<Score>>,
        score_viewport: ScoreViewport,
        player: Arc<Mutex<Player>>,
        input_tx: mpsc::Sender<InputEvent>,
        cursor: Cursor,
        selection_buffer: SelectionBuffer,
        loop_state: LoopState,
        instrument_id: u32,  // Add instrument ID parameter
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
            instrument_id,  // Initialize the instrument ID
        }
    }
}

// Add accessor methods to get cursor and viewport
impl TrackEditorGear {
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }
    
    pub fn viewport(&self) -> ScoreViewport {
        self.score_viewport
    }
}

impl Gear for TrackEditorGear {
    fn name(&self) -> &'static str {
        "Track Editor"
    }
    fn get_draw_component(&self) -> Box<dyn DrawComponent> {
        Box::new(BoxDrawComponent::new(Box::new(
            VSplitDrawComponent::new(
                draw_components::VSplitStyle::HalfWithDivider,
                Box::new(TrackDrawComponent::new(
                    Arc::clone(&self.score),
                    self.player.lock().unwrap().state(),
                    self.score_viewport,
                    self.input_tx.clone(),
                    self.cursor,
                    self.selection_buffer.clone(),
                    self.loop_state,
                    self.instrument_id, // Pass the instrument ID to the TrackDrawComponent
                )),
                Box::new(VSplitDrawComponent::new(
                    draw_components::VSplitStyle::StatusBarNoDivider,
                    Box::new(NullComponent {}),
                    Box::new(StatusBarComponent::new(
                        self.cursor,
                        self.score_viewport,
                        self.loop_state,
                        self.instrument_id, // Pass instrument_id to StatusBarComponent
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
                    error!("Failed to save song: {}", e);
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
                // Pass the current instrument ID for the preview
                player.set_current_instrument_id(self.instrument_id);
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
                // Pass the current instrument ID for the preview
                player.set_current_instrument_id(self.instrument_id);
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
                        let mut score_guard = self.score.lock().unwrap();
                        
                        // Calculate duration based on selection time points
                        let duration = selection_range.time_point_end_b32 - selection_range.time_point_start_b32;
                        score_guard.insert_or_remove(pitch, selection_range.time_point_start_b32, duration, self.instrument_id);
                        
                        // Move cursor to end of selection and clear selection mode
                        self.cursor = self.cursor.end_select();
                    }
                    _ => {
                        // Regular single note insertion
                        self.score.lock().unwrap().insert_or_remove(
                            self.cursor.pitch(),
                            self.cursor.time_point(),
                            self.score_viewport.resolution.duration_b32(),
                            self.instrument_id,
                        );
                        self.cursor = self.cursor.right(self.score_viewport.resolution.duration_b32());
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
                    let selection_score = self.score.lock().unwrap().clone_at_selection(selection_range, Some(self.instrument_id));
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
                    let selection_score = self.score.lock().unwrap().clone_at_selection(selection_range, Some(self.instrument_id));
                    self.score.lock().unwrap().delete_in_selection(selection_range, Some(self.instrument_id));
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
                            // Only insert notes matching the current instrument_id
                            if note.instrument_id == self.instrument_id {
                                // Use insert to properly handle note merging
                                score_guard.insert(note.pitch, onset_b32, note.duration_b32, note.instrument_id);
                            }
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
                    self.score.lock().unwrap().delete_in_selection(selection_range, Some(self.instrument_id));
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

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::{Pitch, Tone};
    use std::sync::mpsc;
    use std::collections::HashMap;
    use crate::resolution::Resolution;
    use crate::draw_components::Position;

    fn create_test_gear() -> TrackEditorGear {
        let (tx, _rx) = mpsc::channel();
        let score = Arc::new(Mutex::new(Score {
            bpm: 120,
            notes: HashMap::new(),
            active_notes: HashMap::new(),
        }));
        let player = Arc::new(Mutex::new(Player::create(score.clone(), 44100)));
        
        TrackEditorGear::new(
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
            0, // Test with instrument_id 0
        )
    }

    #[test]
    fn test_gear_creation() {
        let gear = create_test_gear();
        assert_eq!(gear.name(), "Track Editor");
        assert!(gear.viewport_draw_result.is_none());
        assert_eq!(gear.instrument_id, 0);
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
    fn test_navigation_controls() {
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
            let notes = score_guard.notes_starting_at_time(0, Some(0)); // Filter for instrument 0
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
            let notes = score_guard.notes_starting_at_time(0, Some(0));
            assert!(notes.is_empty(), "Note should be removed after second insertion");
        }
    }

    #[test]
    fn test_selection_operations() {
        let mut gear = create_test_gear();
        
        // Set a valid resolution
        gear.score_viewport.resolution = Resolution::Time1_16;
        
        // Add a test note
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
            let notes = score_guard.notes_starting_at_time(0, Some(0));
            assert!(!notes.is_empty(), "Note should be present");
        }
        
        // Test selection and deletion
        gear.cursor = Cursor::new(Pitch::new(Tone::C, 4), 0);
        assert!(gear.handle_event(&InputEvent::SelectIn));
        assert!(gear.handle_event(&InputEvent::CursorRight));
        assert!(gear.handle_event(&InputEvent::Delete));
        
        // Verify note was deleted
        {
            let score_guard = gear.score.lock().unwrap();
            let notes = score_guard.notes_starting_at_time(0, Some(0));
            assert!(notes.is_empty(), "Note should be deleted");
        }
    }

    #[test]
    fn test_copy_paste_operations() {
        let mut gear = create_test_gear();
        
        // Insert a note at time 0
        assert!(gear.handle_event(&InputEvent::InsertNote));
        
        // Move cursor back to time 0 and select the note
        gear.cursor = Cursor::new(Pitch::new(Tone::C, 4), 0);
        assert!(gear.handle_event(&InputEvent::SelectIn));
        assert!(gear.handle_event(&InputEvent::CursorRight));
        
        // Yank (copy) the selection
        assert!(gear.handle_event(&InputEvent::Yank));
        
        // Verify the original note is still there
        {
            let score_guard = gear.score.lock().unwrap();
            let notes = score_guard.notes_starting_at_time(0, Some(0));
            assert_eq!(notes.len(), 1, "Should still have original note");
        }
        
        // Paste at current position
        assert!(gear.handle_event(&InputEvent::Paste));
        
        // Verify both notes exist
        {
            let score_guard = gear.score.lock().unwrap();
            
            let notes_at_0 = score_guard.notes_starting_at_time(0, Some(0));
            assert_eq!(notes_at_0.len(), 1, "Should have original note");
            
            let notes_at_4 = score_guard.notes_starting_at_time(4, Some(0));
            assert_eq!(notes_at_4.len(), 1, "Should have pasted note");
            
            // Verify note properties
            if let Some(note) = notes_at_0.first() {
                assert_eq!(note.onset_b32, 0);
                assert_eq!(note.duration_b32, 2);
                assert_eq!(note.instrument_id, 0);
            }
            
            if let Some(note) = notes_at_4.first() {
                assert_eq!(note.onset_b32, 4);
                assert_eq!(note.duration_b32, 2);
                assert_eq!(note.instrument_id, 0);
            }
        }
    }
}