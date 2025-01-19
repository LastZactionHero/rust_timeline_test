// use crate::score::{Note, Score};
// use std::f64::consts::PI;

// #[derive(PartialEq)]
// enum PlayState {
//     Stopped,
//     Playing,
// }

// pub struct Player {
//     pub score: &'static Score,
//     pub sample_rate: u64,
//     play_state: PlayState,
//     tick: u64,
//     time_b32: u64,
//     active_frequencies: Vec<f64>,
// }

// impl Player {
//     pub fn create(score: &'static Score, sample_rate: u64) -> Player {
//         Player {
//             score,
//             sample_rate,
//             play_state: PlayState::Stopped,
//             tick: 0,
//             time_b32: 0,
//             active_frequencies: vec![],
//         }
//     }

//     pub fn play(&mut self) {
//         self.play_state = PlayState::Playing;
//     }

//     pub fn pause(&mut self) {
//         self.play_state = PlayState::Stopped;
//     }

//     pub fn stop(&mut self) {
//         self.play_state = PlayState::Stopped;
//         self.time_b32 = 0;
//         self.tick = 0;
//     }

//     pub fn toggle_playback(&mut self) {
//         self.play_state = match self.play_state {
//             PlayState::Stopped => PlayState::Playing,
//             PlayState::Playing => PlayState::Stopped,
//         }
//     }
// }

// impl Iterator for Player {
//     type Item = f64;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.play_state == PlayState::Stopped {
//             return Some(0.0);
//         }

//         if self.tick == 0 || self.tick % 1378 == 0 {
//             if self.score.time_within_song(self.time_b32) {
//                 self.active_frequencies = self
//                     .score
//                     .active_notes_at_time(self.time_b32)
//                     .iter()
//                     .map(|note| note.pitch.frequency(note.octave))
//                     .collect();
//             } else {
//                 self.active_frequencies = vec![];
//                 self.stop();
//             }
//         }
//         if self.tick != 0 && self.tick % 1378 == 0 {
//             self.time_b32 += 1;
//         }
//         self.tick += 1;

//         if self.active_frequencies.is_empty() {
//             return Some(0.0);
//         }

//         let mut total_amplitudes: f64 = 0.0;
//         for frequency in &self.active_frequencies {
//             total_amplitudes += (2.0 * PI * frequency * (self.tick as f64) / 44100.0).sin();
//         }

//         Some(total_amplitudes / self.active_frequencies.len() as f64)
//     }
// }
