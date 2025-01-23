// main.rs
use std::{
    io,
    sync::{Arc, Mutex},
};
mod app_state;
mod draw_components;
mod draw_score;
mod events;
mod pitch;
mod player;
mod score;
mod sin_wave;
mod song;

use app_state::AppState;
use song::create_song;

fn main() -> io::Result<()> {
    let score = Arc::new(Mutex::new(create_song()));

    let mut app_state = AppState::new(score);
    app_state.run()?;

    Ok(())
}
