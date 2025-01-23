use crate::score::{Note, Score};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Clone, Copy)]
pub enum PlayState {
    Stopped,
    Playing,
    Paused,
}

pub struct Player {
    score: Arc<Mutex<Score>>,
    sample_rate: u64,
    state: PlayState,
    tick: u64,
    time_b32: u64,
    active_notes: Vec<Note>,
    ticks_per_b32: u64,
}

impl Player {
    pub fn create(score: Arc<Mutex<Score>>, sample_rate: u64) -> Player {
        // Calculate ticks per b32 based on sample rate
        // For 120 BPM: 44100 samples/sec * 60 sec/min / 120 beats/min / 32 subdivisions = 689.0625 samples/b32
        // Rounding to 689 samples per b32 unit
        let ticks_per_b32 = (sample_rate * 60 / score.lock().unwrap().bpm as u64) / 32;

        Player {
            score,
            sample_rate,
            state: PlayState::Stopped,
            tick: 0,
            time_b32: 0,
            active_notes: Vec::new(),
            ticks_per_b32,
        }
    }

    pub fn play(&mut self) {
        self.state = PlayState::Playing;
    }

    pub fn pause(&mut self) {
        self.state = PlayState::Paused;
    }

    pub fn stop(&mut self) {
        self.state = PlayState::Stopped;
        self.time_b32 = 0;
        self.tick = 0;
        self.active_notes.clear();
    }

    pub fn toggle_playback(&mut self) {
        self.state = match self.state {
            PlayState::Playing => PlayState::Paused,
            PlayState::Paused | PlayState::Stopped => PlayState::Playing,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.state == PlayState::Playing
    }

    pub fn current_time_b32(&self) -> u64 {
        self.time_b32
    }

    fn update_active_notes(&mut self) {
        // Get notes starting at current time
        let new_notes = self
            .score
            .lock()
            .unwrap()
            .notes_starting_at_time(self.time_b32);

        // Remove finished notes and add new ones
        self.active_notes
            .retain(|note| note.onset_b32 + note.duration_b32 > self.time_b32);
        self.active_notes.extend(new_notes);
    }
}

impl Iterator for Player {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state != PlayState::Playing {
            return Some(0.0);
        }

        // Update notes when we hit a new b32 boundary
        if self.tick % self.ticks_per_b32 == 0 {
            if self.score.lock().unwrap().time_within_song(self.time_b32) {
                self.update_active_notes();
            } else {
                self.active_notes.clear();
                self.stop();
            }

            if self.tick != 0 {
                self.time_b32 += 1;
            }
        }
        self.tick += 1;

        if self.active_notes.is_empty() {
            return Some(0.0);
        }

        // Generate audio sample from active notes
        let mut total_amplitudes: f64 = 0.0;
        for note in &self.active_notes {
            let frequency = note.pitch.frequency(note.pitch.octave);
            total_amplitudes +=
                (2.0 * PI * frequency * (self.tick as f64) / self.sample_rate as f64).sin();
        }

        Some(total_amplitudes / self.active_notes.len() as f64)
    }
}
