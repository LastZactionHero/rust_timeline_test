// main.rs
use log::*;
use simplelog::*;
use std::fs::File;
use std::{
    io,
    sync::{Arc, Mutex},
};
mod app_state;
mod audio;
mod cursor;
mod draw_components;
mod events;
mod loop_state;
mod mode;
mod pitch;
mod player;
mod resolution;
mod score;
mod score_viewport;
mod selection_buffer;
mod selection_range;
mod sin_wave;
mod song;
mod song_file;

use app_state::AppState;
use crate::score::Score;
use std::collections::HashMap;

fn main() -> io::Result<()> {
    // Initialize logging
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        File::create("debug.log").unwrap(),
    )])
    .unwrap();

    info!("Application starting...");

    let score = Arc::new(Mutex::new(Score {
        bpm: 120,
        notes: HashMap::new(),
        active_notes: HashMap::new(),
    }));
    
    let mut app_state = AppState::new(score);
    app_state.run()?;

    Ok(())
}
