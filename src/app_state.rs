// app_state.rs
use crate::audio::audio_player;
use crate::cursor::Cursor;
use crate::draw_components::ViewportDrawResult;
use crate::gear::{Gear, GearType, TrackEditorGear, ScoreEditorGear};
use crate::loop_state::LoopState;
use crate::pitch::{Pitch, Tone};
use crate::player::Player;
use crate::resolution::Resolution;
use crate::score::Score;
use crate::score_viewport::ScoreViewport;
use crate::{
    draw_components::{
        DrawComponent, DrawResult, Position, Window,
    },
};
use crate::{
    events::{capture_input, InputEvent},
    selection_buffer::SelectionBuffer,
};
use crossterm::{
    cursor::{self},
    style::{self},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use crate::song_file::SongFile;
use log::error;

pub struct AppState {
    score: Arc<Mutex<Score>>,
    score_viewport: ScoreViewport,
    player: Arc<Mutex<Player>>,
    input_tx: mpsc::Sender<InputEvent>,
    input_rx: mpsc::Receiver<InputEvent>,
    input_thread: Option<JoinHandle<()>>,
    audio_thread: Option<JoinHandle<()>>,
    buffer: Option<Vec<Vec<char>>>,
    cursor: Cursor,
    selection_buffer: SelectionBuffer,
    viewport_draw_result: Option<ViewportDrawResult>,
    loop_state: LoopState,
    song_file: SongFile,
    active_gear: Box<dyn Gear>,
    active_gear_type: GearType,
    current_instrument_id: u32, // Track the current instrument ID
}

impl AppState {
    pub fn new(score: Arc<Mutex<Score>>) -> AppState {
        let (tx, rx) = mpsc::channel();

        let player = Player::create(Arc::clone(&score), 44100);
        let shared_player = Arc::new(Mutex::new(player));
        
        let score_viewport = ScoreViewport::new(Pitch::new(Tone::C, 4), Resolution::Time1_16, 0, 0);
        let cursor = Cursor::new(Pitch::new(Tone::C, 4), 0);
        let selection_buffer = SelectionBuffer::None;
        let loop_state = LoopState::new();
        
        // Initialize the track editor gear as the default active gear
        let track_editor = TrackEditorGear::new(
            Arc::clone(&score),
            score_viewport,
            Arc::clone(&shared_player),
            tx.clone(),
            cursor,
            selection_buffer.clone(),
            loop_state,
            0, // Default instrument ID is 0
        );

        AppState {
            score,
            score_viewport,
            player: shared_player,
            input_tx: tx,
            input_rx: rx,
            input_thread: None,
            audio_thread: None,
            buffer: None,
            cursor,
            selection_buffer,
            viewport_draw_result: None,
            loop_state,
            song_file: SongFile::new(),
            active_gear: Box::new(track_editor),
            active_gear_type: GearType::TrackEditor,
            current_instrument_id: 0, // Initialize with instrument 0
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Setup terminal
        let mut stdout = io::stdout();
        stdout.execute(terminal::Clear(ClearType::All))?;

        // Start input thread
        let input_tx = self.input_tx.clone();
        self.input_thread = Some(thread::spawn(move || {
            let _ = capture_input(&input_tx);
        }));

        // Start audio thread
        let player_tx = self.input_tx.clone();
        let player = Arc::clone(&self.player);
        self.audio_thread = Some(thread::spawn(move || {
            let _ = audio_player(&player, player_tx.clone());
        }));

        // Main loop
        self.draw()?;
        self.event_loop()?;

        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn event_loop(&mut self) -> io::Result<()> {
        loop {
            match self.input_rx.recv() {
                Ok(msg) => {
                    // First check for app-level events that should always be handled here
                    match &msg {
                        InputEvent::Quit => break,
                        
                        // Gear switching
                        InputEvent::SwitchGear(gear_type) => {
                            // We need to dereference and clone the gear_type
                            self.switch_gear((*gear_type).clone());
                        }
                        
                        // Instrument switching
                        InputEvent::SwitchInstrument => {
                            self.switch_instrument();
                        }
                                                
                        // All other events should be passed to the active gear
                        _ => {
                            // If the active gear doesn't handle the event, the score editor gear will handle it
                            let event_handled = self.active_gear.handle_event(&msg);
                            
                            // For specific events that modify the score, ensure we rebuild active_notes
                            if event_handled {
                                match &msg {
                                    InputEvent::Paste | InputEvent::Cut | InputEvent::Delete | InputEvent::InsertNote => {
                                        // Ensure the active_notes are up to date
                                        self.score.lock().unwrap().rebuild_active_notes();
                                    }
                                    _ => {}
                                }
                            }
                            
                            // If the gear handles the event, we need to sync up our app state with the gear's internal state
                            if event_handled {
                                // In a more complete implementation, we would update the app state from the gear after handling the event
                                // For now, we'll just acknowledge that the event was handled
                            }
                        }
                    }
                    
                    self.draw()?;
                }
                Err(e) => {
                    eprintln!("Error in event loop: {e}");
                    break;
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self) -> io::Result<()> {
        let (width, height) = terminal::size()?;
        let mut buffer = vec![vec![' '; width as usize]; height as usize];

        let mut stdout = io::stdout();
        if self.buffer.is_none() {
            stdout.execute(terminal::Clear(ClearType::All))?;
        }

        // Get the active gear's draw component
        let gear_component = self.active_gear.get_draw_component();
        
        // Create the base component
        let base_component = Window::new(vec![gear_component]);

        let position = Position {
            x: 0,
            y: 0,
            w: width as usize,
            h: height as usize,
        };
        let draw_results = base_component.draw(&mut buffer, &position);
        for draw_result in draw_results {
            match draw_result {
                DrawResult::ViewportDrawResult(viewport_draw_result) => {
                    // Store the viewport draw result in AppState
                    self.viewport_draw_result = Some(viewport_draw_result);
                    
                    // Also pass it to the active gear
                    self.active_gear.set_viewport_draw_result(viewport_draw_result);
                    
                    let player = self.player.lock().unwrap();
                    if player.is_playing()
                        && (player.current_time_b32() < viewport_draw_result.time_point_start
                            || player.current_time_b32() >= viewport_draw_result.time_point_end)
                    {
                        let new_time = player.current_time_b32() - player.current_time_b32() % 32;
                        self.score_viewport = self.score_viewport.set_time_point(new_time);
                    }
                    
                    if self.cursor.time_point() < viewport_draw_result.time_point_start
                        || self.cursor.time_point() >= viewport_draw_result.time_point_end - 2
                    {
                        let new_time = self.cursor.time_point() - self.cursor.time_point() % 32;
                        self.score_viewport = self.score_viewport.set_time_point(new_time);
                    }
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                let char = buffer[y as usize][x as usize];
                if self.buffer.is_none()
                    || char != self.buffer.as_ref().unwrap()[y as usize][x as usize]
                {
                    stdout
                        .queue(cursor::MoveTo(x, y))?
                        .queue(style::Print(char))?;
                }
            }
        }
        stdout.flush()?;

        self.buffer = Some(buffer);
        Ok(())
    }
    
    /// Switch to the specified gear type
    pub fn switch_gear(&mut self, gear_type: GearType) {
        match gear_type {
            GearType::TrackEditor => {
                let track_editor = TrackEditorGear::new(
                    Arc::clone(&self.score),
                    self.score_viewport,
                    Arc::clone(&self.player),
                    self.input_tx.clone(),
                    self.cursor,
                    self.selection_buffer.clone(),
                    self.loop_state,
                    self.current_instrument_id, // Use current instrument ID
                );
                self.active_gear = Box::new(track_editor);
                self.active_gear_type = GearType::TrackEditor;
                
                // After switching, update the selection buffer from the gear
                self.update_selection_buffer_from_gear();
            }
            GearType::ScoreEditor => {
                let score_editor = ScoreEditorGear::new(
                    Arc::clone(&self.score),
                    self.score_viewport,
                    Arc::clone(&self.player),
                    self.input_tx.clone(),
                    self.cursor,
                    self.selection_buffer.clone(),
                    self.loop_state,
                );
                self.active_gear = Box::new(score_editor);
                self.active_gear_type = GearType::ScoreEditor;
                
                // After switching, update the selection buffer from the gear
                self.update_selection_buffer_from_gear();
            }
            GearType::Mixer => {
                // In the future, implement Mixer gear
                // For now, just log that we can't switch to it yet
                error!("Mixer gear not yet implemented");
            }
        }
    }
    
    /// Helper method to update the AppState's selection buffer from the active gear
    fn update_selection_buffer_from_gear(&mut self) {
        // For now, we just need to make sure we rebuild active_notes
        // We could extend this in the future to sync more state between gear and app
        self.score.lock().unwrap().rebuild_active_notes();
    }
    
    /// Switch to the next instrument or score editor (cycling through instruments 0-3, then score editor)
    pub fn switch_instrument(&mut self) {
        // Cycle through instruments 0-3, then switch to score editor
        if self.active_gear_type == GearType::ScoreEditor {
            // If we're in the score editor, go back to instrument 0
            self.current_instrument_id = 0;
            self.active_gear_type = GearType::TrackEditor;
            
            // Create a new TrackEditorGear
            let track_editor = TrackEditorGear::new(
                Arc::clone(&self.score),
                self.score_viewport,
                Arc::clone(&self.player),
                self.input_tx.clone(),
                self.cursor,
                self.selection_buffer.clone(),
                self.loop_state,
                self.current_instrument_id,
            );
            
            // Update the active gear
            self.active_gear = Box::new(track_editor);
            
            // After switching, update the selection buffer from the gear
            self.update_selection_buffer_from_gear();
        } else {
            // We're in track editor mode
            self.current_instrument_id = (self.current_instrument_id + 1) % 4;
            
            if self.current_instrument_id == 0 {
                // After cycling through all instruments, switch to score editor
                self.active_gear_type = GearType::ScoreEditor;
                
                // Create a new ScoreEditorGear
                let score_editor = ScoreEditorGear::new(
                    Arc::clone(&self.score),
                    self.score_viewport,
                    Arc::clone(&self.player),
                    self.input_tx.clone(),
                    self.cursor,
                    self.selection_buffer.clone(),
                    self.loop_state,
                );
                
                // Update the active gear
                self.active_gear = Box::new(score_editor);
                
                // After switching, update the selection buffer from the gear
                self.update_selection_buffer_from_gear();
            } else {
                // Switch to next instrument track
                let track_editor = TrackEditorGear::new(
                    Arc::clone(&self.score),
                    self.score_viewport,
                    Arc::clone(&self.player),
                    self.input_tx.clone(),
                    self.cursor,
                    self.selection_buffer.clone(),
                    self.loop_state,
                    self.current_instrument_id,
                );
                
                // Update the active gear
                self.active_gear = Box::new(track_editor);
                
                // After switching, update the selection buffer from the gear
                self.update_selection_buffer_from_gear();
            }
        }
    }
}
