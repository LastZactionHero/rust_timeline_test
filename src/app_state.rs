// app_state.rs
use crate::audio::audio_player;
use crate::cursor::Cursor;
use crate::draw_components::ViewportDrawResult;
use crate::gear::{Gear, GearType, ScoreEditorGear};
use crate::loop_state::LoopState;
use crate::pitch::{Pitch, Tone};
use crate::player::Player;
use crate::resolution::Resolution;
use crate::score::Score;
use crate::score_viewport::ScoreViewport;
use crate::{
    draw_components::{
        self, BoxDrawComponent, DrawComponent, DrawResult, NullComponent, Position, VSplitDrawComponent,
        Window,
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
        
        // Initialize the score editor gear as the default active gear
        let score_editor = ScoreEditorGear::new(
            Arc::clone(&score),
            score_viewport,
            Arc::clone(&shared_player),
            tx.clone(),
            cursor,
            selection_buffer.clone(),
            loop_state,
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
            active_gear: Box::new(score_editor),
            active_gear_type: GearType::ScoreEditor,
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
                        
                        // All editor-specific operations now handled by ScoreEditorGear
                        
                        // All other events should be passed to the active gear
                        _ => {
                            // If the active gear doesn't handle the event, the score editor gear will handle it
                            let event_handled = self.active_gear.handle_event(&msg);
                            
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
                    self.viewport_draw_result = Some(viewport_draw_result);
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
            }
            GearType::Mixer => {
                // In the future, implement Mixer gear
                // For now, just log that we can't switch to it yet
                error!("Mixer gear not yet implemented");
            }
        }
    }
}
