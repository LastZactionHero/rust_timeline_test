// main.rs
use std::io;
mod app_state;
mod draw_components;
mod draw_score;
mod pitch;
mod player;
mod score;
mod sin_wave;
mod song;

use app_state::AppState;
use score::Score;
use song::create_song;

fn main() -> io::Result<()> {
    let score = create_song();
    let score: &'static Score = Box::leak(Box::new(score));

    let mut app_state = AppState::new(score);
    app_state.run()?;

    Ok(())
}
