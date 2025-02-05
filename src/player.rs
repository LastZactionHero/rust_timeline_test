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

pub struct FMVoice {
    carrier_freq: f64,
    modulator_freq: f64,
    modulation_index: f64,
    start_tick: u64,
    duration_ticks: u64,
}

pub struct Player {
    score: Arc<Mutex<Score>>,
    sample_rate: u64,
    state: PlayState,
    tick: u64,
    time_b32: u64,
    active_notes: Vec<Note>,
    fm_voices: Vec<FMVoice>,
    ticks_per_b32: u64,
    // FM synthesis parameters
    modulation_ratio: f64,
    modulation_index: f64,
}

impl Player {
    pub fn create(score: Arc<Mutex<Score>>, sample_rate: u64) -> Player {
        let ticks_per_b32 = (sample_rate * 60 / score.lock().unwrap().bpm as u64) / 32;

        Player {
            score,
            sample_rate,
            state: PlayState::Stopped,
            tick: 0,
            time_b32: 0,
            active_notes: Vec::new(),
            fm_voices: Vec::new(),
            ticks_per_b32,
            modulation_ratio: 2.0, // Default ratio for metallic sounds
            modulation_index: 5.0, // Default modulation depth
        }
    }

    // Add setters for FM parameters
    pub fn set_modulation_ratio(&mut self, ratio: f64) {
        self.modulation_ratio = ratio;
    }

    pub fn set_modulation_index(&mut self, index: f64) {
        self.modulation_index = index;
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
        self.fm_voices.clear();
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

        // Remove finished notes
        self.active_notes
            .retain(|note| note.onset_b32 + note.duration_b32 > self.time_b32);

        // Add new FM voices for new notes
        for note in new_notes.iter() {
            let carrier_freq = note.pitch.frequency(note.pitch.octave);
            let modulator_freq = carrier_freq * self.modulation_ratio;
            let duration_ticks = (note.duration_b32 * self.ticks_per_b32) as u64;

            self.fm_voices.push(FMVoice {
                carrier_freq,
                modulator_freq,
                modulation_index: self.modulation_index,
                start_tick: self.tick,
                duration_ticks,
            });
        }

        // Add new notes to active notes
        self.active_notes.extend(new_notes);

        // Clean up finished FM voices
        self.fm_voices
            .retain(|voice| (self.tick - voice.start_tick) < voice.duration_ticks);
    }

    pub fn state(&self) -> PlayState {
        self.state
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
                self.fm_voices.clear();
                self.stop();
            }

            if self.tick != 0 {
                self.time_b32 += 1;
            }
        }
        self.tick += 1;

        if self.fm_voices.is_empty() {
            return Some(0.0);
        }

        // Generate FM synthesis sample
        let mut total_sample: f64 = 0.0;
        let time = self.tick as f64 / self.sample_rate as f64;

        for voice in &self.fm_voices {
            // Calculate the modulator's instantaneous value
            let modulator = (2.0 * PI * voice.modulator_freq * time).sin();

            // Apply the modulator to the carrier frequency
            let carrier_phase =
                2.0 * PI * voice.carrier_freq * time + voice.modulation_index * modulator;

            // Generate the final FM waveform
            total_sample += carrier_phase.sin();
        }

        // Normalize output
        Some(total_sample / self.fm_voices.len() as f64)
    }
}
