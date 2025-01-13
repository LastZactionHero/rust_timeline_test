use crate::score::{Note, Score};

enum PlayState {
    Stopped,
    Playing,
}

pub struct Player<'a> {
    pub score: &'a Score,
    pub sample_rate: u64,
    play_state: PlayState,
    time_b32: u64,
}

impl<'a> Player<'a> {
    pub fn create(score: &'a Score, sample_rate: u64) -> Player<'a> {
        Player {
            score,
            sample_rate,
            play_state: PlayState::Stopped,
            time_b32: 0,
        }
    }

    pub fn play(&mut self) {
        self.play_state = PlayState::Playing;
    }

    pub fn pause(&mut self) {
        self.play_state = PlayState::Stopped;
    }

    pub fn stop(&mut self) {
        self.play_state = PlayState::Stopped;
        self.time_b32 = 0;
    }

    pub fn toggle_playback(&mut self) {
        self.play_state = match self.play_state {
            PlayState::Stopped => PlayState::Playing,
            PlayState::Playing => PlayState::Stopped,
        }
    }
}

impl<'a> Iterator for Player<'a> {
    type Item = Vec<Note>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_time_b32 = self.time_b32 + 1;
        if self.score.time_within_song(next_time_b32) {
            self.time_b32 = next_time_b32;
            return Some(self.score.active_notes_at_time(self.time_b32));
        }
        None
    }
}
