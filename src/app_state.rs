// app_state.rs
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossterm::{
    cursor::{self},
    event::{poll, read, Event, KeyCode},
    style::{self},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::{
    io::{self, Write},
    time::Duration,
};

use crate::draw_components::{
    self,
    score_draw_component::{Resolution, ScoreDrawComponent, ScoreViewport},
    BoxDrawComponent, DrawComponent, NullComponent, Position, Window,
};
use crate::pitch::{Pitch, Tone};
use crate::player::Player;
use crate::score::Score;

pub enum InputEvent {
    ViewerOctaveIncrease,
    ViewerOctaveDecrease,
    ViewerBarNext,
    ViewerBarPrevious,
    ViewerResolutionIncrease,
    ViewerResolutionDecrease,
    PlayerTogglePlayback,
    Quit,
}

pub struct AppState {
    score: &'static Score,
    score_viewport: ScoreViewport,
    player: Arc<Mutex<Player>>,
    input_tx: mpsc::Sender<InputEvent>,
    input_rx: mpsc::Receiver<InputEvent>,
    input_thread: Option<JoinHandle<()>>,
    audio_thread: Option<JoinHandle<()>>,
}

impl AppState {
    pub fn new(score: &'static Score) -> AppState {
        let player = Player::create(score, 44100);
        let shared_player = Arc::new(Mutex::new(player));
        let (tx, rx) = mpsc::channel();

        AppState {
            score,
            score_viewport: ScoreViewport::new(Pitch::new(Tone::C, 4), Resolution::Time1_16, 0),
            player: shared_player,
            input_tx: tx,
            input_rx: rx,
            input_thread: None,
            audio_thread: None,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Setup terminal
        let mut stdout = io::stdout();
        stdout.execute(terminal::Clear(ClearType::All))?;

        // Start input thread
        let input_tx = self.input_tx.clone();
        self.input_thread = Some(thread::spawn(move || {
            let _ = Self::capture_input(&input_tx);
        }));

        // Start audio thread
        let player = Arc::clone(&self.player);
        self.audio_thread = Some(thread::spawn(move || {
            let _ = Self::audio_player(player);
        }));

        // Main loop
        self.draw()?;
        self.event_loop()?;

        Ok(())
    }

    fn event_loop(&mut self) -> io::Result<()> {
        loop {
            match self.input_rx.recv() {
                Ok(msg) => {
                    match msg {
                        InputEvent::Quit => break,
                        InputEvent::ViewerOctaveIncrease => {
                            if self.score_viewport.middle_pitch.octave < 8 {
                                self.score_viewport.middle_pitch.octave += 1;
                            }
                        }
                        InputEvent::ViewerOctaveDecrease => {
                            if self.score_viewport.middle_pitch.octave > 0 {
                                self.score_viewport.middle_pitch.octave -= 1;
                            }
                        }
                        InputEvent::ViewerBarNext => {
                            self.score_viewport.time_point += 1;
                        }
                        InputEvent::ViewerBarPrevious => {
                            if self.score_viewport.time_point > 1 {
                                self.score_viewport.time_point -= 1;
                            }
                        }
                        InputEvent::ViewerResolutionIncrease => {
                            self.score_viewport.resolution =
                                self.score_viewport.resolution.next_up();
                        }
                        InputEvent::ViewerResolutionDecrease => {
                            self.score_viewport.resolution =
                                self.score_viewport.resolution.next_down();
                        }
                        InputEvent::PlayerTogglePlayback => {
                            let mut player_guard = self.player.lock().unwrap();
                            player_guard.toggle_playback();
                        }
                    }
                    self.draw()?;
                }
                Err(e) => {
                    eprintln!("Error in event loop: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }

    fn draw(&self) -> io::Result<()> {
        let (width, height) = terminal::size()?;
        let mut buffer = vec![vec![' '; width as usize]; height as usize];

        let mut stdout = io::stdout();
        stdout.execute(terminal::Clear(ClearType::All))?;

        let base_component = Window::new(vec![Box::new(BoxDrawComponent::new(Box::new(
            draw_components::VSplitDrawComponent::new(
                Box::new(ScoreDrawComponent::new(
                    self.score,
                    self.score_viewport.clone(),
                )),
                Box::new(draw_components::FillComponent { value: '0' }),
            ),
        )))]);

        let position = Position {
            x: 0,
            y: 0,
            w: width as usize,
            h: height as usize,
        };
        base_component.draw(&mut buffer, &position);

        for y in 0..height {
            let row: String = buffer[y as usize].clone().into_iter().collect();
            stdout
                .queue(cursor::MoveTo(0, y))?
                .queue(style::Print(row))?;
        }
        stdout.flush()?;

        Ok(())
    }

    fn capture_input(tx: &mpsc::Sender<InputEvent>) -> io::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        loop {
            if poll(Duration::from_millis(500))? {
                if let Event::Key(event) = read()? {
                    match event.code {
                        KeyCode::Char('q') => {
                            tx.send(InputEvent::Quit).unwrap();
                            break;
                        }
                        KeyCode::Up => tx.send(InputEvent::ViewerOctaveIncrease).unwrap(),
                        KeyCode::Down => tx.send(InputEvent::ViewerOctaveDecrease).unwrap(),
                        KeyCode::Left => tx.send(InputEvent::ViewerBarPrevious).unwrap(),
                        KeyCode::Right => tx.send(InputEvent::ViewerBarNext).unwrap(),
                        KeyCode::Char('[') => {
                            tx.send(InputEvent::ViewerResolutionDecrease).unwrap()
                        }
                        KeyCode::Char(']') => {
                            tx.send(InputEvent::ViewerResolutionIncrease).unwrap()
                        }
                        KeyCode::Char(' ') => tx.send(InputEvent::PlayerTogglePlayback).unwrap(),
                        _ => (),
                    }
                }
            }
        }
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    fn audio_player(player: Arc<Mutex<Player>>) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("Did not find default output device");
        let config = device.default_output_config().unwrap();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
        let stream_config: cpal::StreamConfig = config.into();
        let channels = stream_config.channels as usize;

        let player_clone = player.clone();
        let stream = device.build_output_stream(
            &stream_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                Self::write_data(data, channels, player_clone.clone())
            },
            err_fn,
            None,
        )?;
        stream.play()?;

        loop {
            thread::sleep(Duration::from_millis(1000));
        }
    }

    fn write_data(output: &mut [f32], channels: usize, player: Arc<Mutex<Player>>) {
        for frame in output.chunks_mut(channels) {
            let sample = player.lock().unwrap().next().unwrap() as f32;
            for s in frame.iter_mut() {
                *s = sample;
            }
        }
    }
}
