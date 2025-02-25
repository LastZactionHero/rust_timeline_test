use std::sync::{Arc, Mutex, mpsc};

use crate::cursor::Cursor;
use crate::cursor::CursorMode;
use crate::draw_components::{DrawComponent, BoxDrawComponent, VSplitDrawComponent, NullComponent};
use crate::draw_components::score_draw_component::ScoreDrawComponent;
use crate::draw_components::status_bar_component::StatusBarComponent;
use crate::draw_components::{self, DrawResult, ViewportDrawResult};
use crate::events::InputEvent;
use crate::loop_state::LoopState;
use crate::player::Player;
use crate::resolution::Resolution;
use crate::score::Score;
use crate::score_viewport::ScoreViewport;
use crate::selection_buffer::SelectionBuffer;
use crate::song_file::SongFile;
use log::error;

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
                    )),
                )),
            ),
        )))
    }
    fn set_viewport_draw_result(&mut self, result: ViewportDrawResult) {
        self.viewport_draw_result = Some(result);
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
                self.player.lock().unwrap().preview_note(self.cursor.pitch());
                true
            }
            InputEvent::CursorDown => {
                self.cursor = self.cursor.down();
                match self.score_viewport.middle_pitch.prev() {
                    Some(prev_pitch) => self.score_viewport.middle_pitch = prev_pitch,
                    None => (),
                }
                self.player.lock().unwrap().preview_note(self.cursor.pitch());
                true
            }
            InputEvent::CursorLeft => {
                self.cursor = self.cursor.left(self.score_viewport.resolution.duration_b32());
                self.selection_buffer = self.selection_buffer.translate_to(self.cursor.time_point());
                true
            }
            InputEvent::CursorRight => {
                self.cursor = self.cursor.right(self.score_viewport.resolution.duration_b32());
                self.selection_buffer = self.selection_buffer.translate_to(self.cursor.time_point());
                true
            }
            
            // Note editing
            InputEvent::InsertNote => {
                match self.cursor.mode() {
                    CursorMode::Select(start, end) => {
                        // Insert notes for the entire selection
                        let selection_range = self.cursor.selection_range().unwrap();
                        let pitch = self.cursor.pitch();
                        let mut score_guard = self.score.lock().unwrap();
                        
                        // Calculate duration based on selection time points
                        let duration = selection_range.time_point_end_b32 - selection_range.time_point_start_b32;
                        score_guard.insert_or_remove(pitch, selection_range.time_point_start_b32, duration);
                        
                        // Move cursor to end of selection and clear selection mode
                        self.cursor = self.cursor.end_select();
                    }
                    _ => {
                        // Regular single note insertion
                        self.score.lock().unwrap().insert_or_remove(
                            self.cursor.pitch(),
                            self.cursor.time_point(),
                            self.score_viewport.resolution.duration_b32(),
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
                    let selection_score = self.score.lock().unwrap().clone_at_selection(selection_range);
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
                    let selection_score = self.score.lock().unwrap().clone_at_selection(selection_range);
                    self.score.lock().unwrap().delete_in_selection(selection_range);
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
                    *score_guard = score_guard.merge_down(selection_buffer_score);
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
                    self.score.lock().unwrap().delete_in_selection(selection_range);
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