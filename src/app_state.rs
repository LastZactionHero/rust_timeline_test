// app_state.rs
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossterm::{
    cursor::{self},
    style::{self},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::thread::{self, JoinHandle};
use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
};
use std::{
    io::{self, Write},
    time::Duration,
};

use crate::mode::Mode;
use crate::pitch::{Pitch, Tone};
use crate::player::Player;
use crate::resolution::Resolution;
use crate::score::Score;
use crate::{cursor::Cursor, draw_components::ViewportDrawResult};
use crate::{
    cursor::CursorMode,
    draw_components::{
        self,
        score_draw_component::{ScoreDrawComponent, ScoreViewport},
        status_bar_component::StatusBarComponent,
        BoxDrawComponent, DrawComponent, DrawResult, FillComponent, NullComponent, Position,
        VSplitDrawComponent, Window,
    },
};
use crate::{
    events::{capture_input, InputEvent},
    selection_buffer::SelectionBuffer,
};

pub struct AppState {
    score: Arc<Mutex<Score>>,
    score_viewport: ScoreViewport,
    player: Arc<Mutex<Player>>,
    input_tx: mpsc::Sender<InputEvent>,
    input_rx: mpsc::Receiver<InputEvent>,
    input_thread: Option<JoinHandle<()>>,
    audio_thread: Option<JoinHandle<()>>,
    buffer: Option<Vec<Vec<char>>>,
    mode: Arc<Mutex<Mode>>,
    cursor: Cursor,
    selection_buffer: SelectionBuffer,
}

impl AppState {
    pub fn new(score: Arc<Mutex<Score>>) -> AppState {
        let (tx, rx) = mpsc::channel();

        let player = Player::create(Arc::clone(&score), 44100);
        let shared_player = Arc::new(Mutex::new(player));

        AppState {
            score,
            score_viewport: ScoreViewport::new(Pitch::new(Tone::C, 4), Resolution::Time1_16, 0, 0),
            player: shared_player,
            input_tx: tx,
            input_rx: rx,
            input_thread: None,
            audio_thread: None,
            buffer: None,
            mode: Arc::new(Mutex::new(Mode::Normal)),
            cursor: Cursor::new(Pitch::new(Tone::C, 4), 0),
            selection_buffer: SelectionBuffer::None,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Setup terminal
        let mut stdout = io::stdout();
        stdout.execute(terminal::Clear(ClearType::All))?;

        // Start input thread
        let input_tx = self.input_tx.clone();
        let mode_clone = Arc::clone(&self.mode);
        self.input_thread = Some(thread::spawn(move || {
            let _ = capture_input(&input_tx, &mode_clone);
        }));

        // Start audio thread
        let player_tx = self.input_tx.clone();
        let player = Arc::clone(&self.player);
        self.audio_thread = Some(thread::spawn(move || {
            let _ = Self::audio_player(&player, player_tx.clone());
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
                    match msg {
                        InputEvent::Quit => break,
                        InputEvent::ViewerOctaveIncrease => {
                            if let Some(next_pitch) = self.score_viewport.middle_pitch.next() {
                                self.score_viewport.middle_pitch = next_pitch;
                            }
                        }
                        InputEvent::ViewerOctaveDecrease => {
                            if let Some(prev_pitch) = self.score_viewport.middle_pitch.prev() {
                                self.score_viewport.middle_pitch = prev_pitch;
                            }
                        }
                        InputEvent::ViewerBarNext => {
                            self.score_viewport.time_point += 32;
                        }
                        InputEvent::ViewerBarPrevious => {
                            if self.score_viewport.time_point > 0 {
                                self.score_viewport.time_point -= 32;
                            }
                        }
                        InputEvent::ViewerResolutionIncrease => {
                            self.score_viewport.resolution =
                                self.score_viewport.resolution.next_up();
                            self.cursor = self
                                .cursor
                                .resolution_align(self.score_viewport.resolution.duration_b32());
                        }
                        InputEvent::ViewerResolutionDecrease => {
                            self.score_viewport.resolution =
                                self.score_viewport.resolution.next_down();
                            self.cursor = self
                                .cursor
                                .resolution_align(self.score_viewport.resolution.duration_b32());
                        }
                        InputEvent::PlayerTogglePlayback => {
                            let mut player_guard = self.player.lock().unwrap();
                            player_guard.toggle_playback();
                        }
                        InputEvent::PlayerBeatChange(playback_time_point_b32) => {
                            self.score_viewport.playback_time_point = playback_time_point_b32;
                        }
                        InputEvent::ToggleMode => {
                            let next_mode = match *self.mode.lock().unwrap() {
                                Mode::Normal => Mode::Insert,
                                Mode::Insert => Mode::Select,
                                Mode::Select => Mode::Normal,
                            };
                            *self.mode.lock().unwrap() = next_mode;
                            self.cursor = match *self.mode.lock().unwrap() {
                                Mode::Select | Mode::Insert => self.cursor.show(),
                                Mode::Normal => self.cursor.hide(),
                            };
                        }
                        InputEvent::CursorUp => {
                            self.cursor = self.cursor.up();
                            match self.score_viewport.middle_pitch.next() {
                                Some(next_pitch) => self.score_viewport.middle_pitch = next_pitch,
                                None => (),
                            }
                        }
                        InputEvent::CursorDown => {
                            self.cursor = self.cursor.down();
                            match self.score_viewport.middle_pitch.prev() {
                                Some(prev_pitch) => self.score_viewport.middle_pitch = prev_pitch,
                                None => (),
                            }
                        }
                        InputEvent::CursorLeft => {
                            self.cursor = self
                                .cursor
                                .left(self.score_viewport.resolution.duration_b32());
                        }
                        InputEvent::CursorRight => {
                            self.cursor = self
                                .cursor
                                .right(self.score_viewport.resolution.duration_b32());
                        }
                        InputEvent::InsertNoteAtCursor => {
                            self.score.lock().unwrap().insert_or_remove(
                                self.cursor.pitch(),
                                self.cursor.time_point(),
                                self.score_viewport.resolution.duration_b32(),
                            );
                            self.cursor = self
                                .cursor
                                .right(self.score_viewport.resolution.duration_b32());
                        }
                        InputEvent::StartNoteAtCursor => match self.cursor.mode() {
                            CursorMode::Move => match *self.mode.lock().unwrap() {
                                Mode::Select => {
                                    self.cursor = self.cursor.start_select();
                                }
                                Mode::Insert => {
                                    self.cursor = self.cursor.start_insert();
                                }
                                Mode::Normal => {}
                            },
                            CursorMode::Insert(onset_b32) => {
                                if onset_b32 < self.cursor.time_point() {
                                    self.score.lock().unwrap().insert_or_remove(
                                        self.cursor.pitch(),
                                        onset_b32,
                                        self.cursor.time_point() - onset_b32 + 2,
                                    );
                                }
                                self.cursor = self.cursor.end_insert();
                                self.cursor = self
                                    .cursor
                                    .right(self.score_viewport.resolution.duration_b32());
                            }
                            CursorMode::Select(pitch, onset_b32) => {
                                self.cursor = self.cursor.end_select();
                            }
                            CursorMode::Yank => {}
                        },
                        InputEvent::Cancel => self.cursor = self.cursor.cancel(),
                        InputEvent::Yank => {
                            // TODO:  Render selection score at the cursor position
                            let selection_range = self.cursor.selection_range().unwrap();
                            self.selection_buffer = SelectionBuffer::Score(
                                self.score.lock().unwrap().clone_at_selection(
                                    selection_range.time_point_start_b32,
                                    selection_range.time_point_end_b32,
                                    selection_range.pitch_low,
                                    selection_range.pitch_high,
                                ),
                            );
                            self.cursor = self.cursor.yank();
                        }
                        InputEvent::Cut => {}
                        InputEvent::Paste => {}
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

        let base_component = Window::new(vec![Box::new(BoxDrawComponent::new(Box::new(
            VSplitDrawComponent::new(
                draw_components::VSplitStyle::HalfWithDivider,
                Box::new(ScoreDrawComponent::new(
                    Arc::clone(&self.score),
                    self.player.lock().unwrap().state(),
                    self.score_viewport,
                    self.input_tx.clone(),
                    self.cursor,
                )),
                Box::new(VSplitDrawComponent::new(
                    draw_components::VSplitStyle::StatusBarNoDivider,
                    Box::new(NullComponent {}),
                    Box::new(StatusBarComponent::new(
                        Arc::clone(&self.mode),
                        self.cursor,
                        self.score_viewport,
                    )),
                )),
            ),
        )))]);

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
                    match *self.mode.lock().unwrap() {
                        Mode::Normal => {
                            let player = self.player.lock().unwrap();

                            if player.is_playing()
                                && (player.current_time_b32()
                                    < viewport_draw_result.time_point_start
                                    || player.current_time_b32()
                                        >= viewport_draw_result.time_point_end)
                            {
                                self.score_viewport.time_point =
                                    player.current_time_b32() - player.current_time_b32() % 32
                            }
                        }
                        Mode::Insert | Mode::Select => {
                            if self.cursor.time_point() < viewport_draw_result.time_point_start
                                || self.cursor.time_point()
                                    >= viewport_draw_result.time_point_end - 2
                            {
                                // TODO: Rework this
                                self.score_viewport.time_point =
                                    self.cursor.time_point() - self.cursor.time_point() % 32;
                            }
                        }
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

    fn audio_player(
        player: &Arc<Mutex<Player>>,
        tx: mpsc::Sender<InputEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("Did not find default output device");
        let config = device.default_output_config().unwrap();

        let err_fn = |err| eprintln!("an error occurred on stream: {err}");
        let stream_config: cpal::StreamConfig = config.into();
        let channels = stream_config.channels as usize;

        let player_clone = player.clone();
        let stream = device.build_output_stream(
            &stream_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                Self::write_data(data, channels, &player_clone.clone(), &tx.clone());
            },
            err_fn,
            None,
        )?;
        stream.play()?;

        loop {
            thread::sleep(Duration::from_millis(1000));
        }
    }

    fn write_data(
        output: &mut [f32],
        channels: usize,
        player: &Arc<Mutex<Player>>,
        tx: &mpsc::Sender<InputEvent>,
    ) {
        let mut time_b32 = player.lock().unwrap().current_time_b32();
        for frame in output.chunks_mut(channels) {
            #[allow(clippy::cast_possible_truncation)]
            let sample = player.lock().unwrap().next().unwrap() as f32;
            let next_time_b32 = player.lock().unwrap().current_time_b32();
            if next_time_b32 != time_b32 {
                time_b32 = next_time_b32;
                tx.send(InputEvent::PlayerBeatChange(time_b32)).unwrap();
            }
            for s in frame.iter_mut() {
                *s = sample;
            }
        }
    }
}
