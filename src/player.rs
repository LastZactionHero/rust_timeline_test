use crate::score::Score;
use crate::score_player::ScorePlayer;
use std::f64::consts::PI;

#[derive(PartialEq)]
enum PlayState {
    Stopped,
    Playing,
}

pub struct Player {
    pub sample_rate: u64,
    score_player: ScorePlayer,
    play_state: PlayState,
    tick: u64,
    active_frequencies: Vec<f64>,
}

impl Player {
    pub fn create(score: &'static Score, sample_rate: u64) -> Player {
        Player {
            sample_rate,
            score_player: ScorePlayer::create(score),
            play_state: PlayState::Stopped,
            tick: 0,
            active_frequencies: vec![],
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
        self.tick = 0;
        self.score_player.reset();
    }

    pub fn toggle_playback(&mut self) {
        self.play_state = match self.play_state {
            PlayState::Stopped => PlayState::Playing,
            PlayState::Playing => PlayState::Stopped,
        }
    }
}

impl Iterator for Player {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.play_state == PlayState::Stopped {
            return Some(0.0);
        }

        if self.tick == 0 || self.tick % (self.sample_rate / 32) == 0 {
            self.active_frequencies = match self.score_player.next() {
                Some(notes) => notes
                    .iter()
                    .map(|note| note.pitch.frequency(note.octave))
                    .collect(),
                None => {
                    self.stop();
                    vec![]
                }
            }
        }
        self.tick += 1;

        if self.active_frequencies.is_empty() {
            return Some(0.0);
        }

        let mut total_amplitudes: f64 = 0.0;
        for frequency in &self.active_frequencies {
            total_amplitudes +=
                (2.0 * PI * frequency * (self.tick as f64) / (self.sample_rate as f64)).sin();
        }

        Some(total_amplitudes / self.active_frequencies.len() as f64)
    }
}
