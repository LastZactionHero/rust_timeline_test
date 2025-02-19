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

use app_state::AppState;
use song::create_song;

fn main() -> io::Result<()> {
    // Initialize logging
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        File::create("debug.log").unwrap(),
    )])
    .unwrap();

    info!("Application starting...");

    let score = Arc::new(Mutex::new(create_song()));
    let mut app_state = AppState::new(score);
    app_state.run()?;

    Ok(())
}
