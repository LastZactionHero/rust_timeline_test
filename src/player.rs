use crate::score::{ActiveNote, Note, Score};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};
use crate::loop_state::LoopState;
use std::time::Instant;
use crate::pitch::Pitch;
use crate::instruments::{Instrument, InstrumentRegistry};

#[derive(PartialEq, Clone, Copy)]
pub enum PlayState {
    Stopped,
    Playing,
    Paused,
    Preview,
}

struct NoteEnvelope {
    note: Note,
    start_tick: u64,
    end_tick: Option<u64>,
}

pub struct Player {
    score: Arc<Mutex<Score>>,
    sample_rate: u64,
    state: PlayState,
    tick: u64,
    time_b32: u64,
    active_notes: Vec<NoteEnvelope>,
    ticks_per_b32: u64,
    loop_state: LoopState,
    preview_start: Option<Instant>,
    instrument_registry: InstrumentRegistry,
    current_instrument_id: u32,  // Track current instrument ID
}

impl Player {
    pub fn create(score: Arc<Mutex<Score>>, sample_rate: u64) -> Player {
        // Calculate ticks per b32 based on sample rate
        // For 120 BPM: 44100 samples/sec * 60 sec/min / 120 beats/min / 32 subdivisions = 689.0625 samples/b32
        // Rounding to 689 samples per b32 unit
        let ticks_per_b32 = (sample_rate * 60 / score.lock().unwrap().bpm as u64) / 32;

        // Create instrument registry with default instruments
        let instrument_registry = InstrumentRegistry::new();

        Player {
            score,
            sample_rate,
            state: PlayState::Stopped,
            tick: 0,
            time_b32: 0,
            active_notes: Vec::new(),
            ticks_per_b32,
            loop_state: LoopState::new(),
            preview_start: None,
            instrument_registry,
            current_instrument_id: 0,  // Default to instrument 0
        }
    }
    
    // Calculate envelope value for a note
    fn calculate_envelope(&self, note_env: &NoteEnvelope) -> f64 {
        // Basic envelope parameters (milliseconds converted to ticks)
        let attack_ticks = (self.sample_rate as f64 * 0.01) as u64; // 10ms attack
        let release_ticks = (self.sample_rate as f64 * 0.03) as u64; // 30ms release
        
        let current_tick = self.tick;
        let note_start_tick = note_env.start_tick;
        
        // If note has an end time, calculate release phase
        if let Some(end_tick) = note_env.end_tick {
            // Note is in release phase
            if current_tick >= end_tick {
                let release_progress = current_tick.saturating_sub(end_tick) as f64 / release_ticks as f64;
                if release_progress >= 1.0 {
                    return 0.0; // Note is completely released
                }
                return 1.0 - release_progress; // Linear release
            }
        }
        
        // Note is in attack phase
        if current_tick < note_start_tick + attack_ticks {
            let attack_progress = (current_tick - note_start_tick) as f64 / attack_ticks as f64;
            return attack_progress; // Linear attack
        }
        
        // Note is in sustain phase
        1.0
    }

    pub fn play(&mut self) {
        self.state = PlayState::Playing;
    }

    pub fn pause(&mut self) {
        self.state = PlayState::Paused;
        
        // Mark all active notes for release
        let current_tick = self.tick;
        for note in &mut self.active_notes {
            if note.end_tick.is_none() {
                note.end_tick = Some(current_tick);
            }
        }
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
            PlayState::Preview => PlayState::Paused,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.state == PlayState::Playing || self.state == PlayState::Preview
    }

    pub fn current_time_b32(&self) -> u64 {
        self.time_b32
    }

    pub fn set_time_b32(&mut self, time_b32: u64) {
        // Mark all notes for release
        let current_tick = self.tick;
        for note in &mut self.active_notes {
            if note.end_tick.is_none() {
                note.end_tick = Some(current_tick);
            }
        }
        
        self.pause();
        self.time_b32 = time_b32;
        self.tick = 0;
        self.update_active_notes();
    }

    pub fn set_loop_state(&mut self, loop_state: LoopState) {
        self.loop_state = loop_state;
    }

    fn update_active_notes(&mut self) {
        // Get notes starting at current time
        let new_notes = self
            .score
            .lock()
            .unwrap()
            .notes_starting_at_time(self.time_b32, None);
            
        // Add new notes with envelopes
        let current_tick = self.tick;
        for note in new_notes {
            self.active_notes.push(NoteEnvelope {
                note,
                start_tick: current_tick,
                end_tick: None,
            });
        }

        // Process completed notes for release
        let time_b32 = self.time_b32;
        for note_env in &mut self.active_notes {
            // If note has completed its duration and isn't already in release phase
            if note_env.note.onset_b32 + note_env.note.duration_b32 <= time_b32 && note_env.end_tick.is_none() {
                note_env.end_tick = Some(current_tick);
            }
        }
        
        // Remove notes that have completed their release phase
        let release_ticks = (self.sample_rate as f64 * 0.03) as u64; // 30ms release
        self.active_notes.retain(|note_env| {
            if let Some(end_tick) = note_env.end_tick {
                // Keep note if it's still in release phase
                current_tick < end_tick + release_ticks
            } else {
                // Keep all notes still in their active duration
                true
            }
        });
    }

    pub fn state(&self) -> PlayState {
        return self.state;
    }

    fn handle_time_update(&mut self) {
        if self.tick != 0 {
            self.time_b32 += 1;
        }

        if self.loop_state.is_looping() {
            if let (Some(start), Some(end)) = (self.loop_state.start_time_b32, self.loop_state.end_time_b32) {
                if self.time_b32 >= end || self.time_b32 < start {
                    // Before looping, mark all active notes for release
                    let current_tick = self.tick;
                    for note in &mut self.active_notes {
                        if note.end_tick.is_none() {
                            note.end_tick = Some(current_tick);
                        }
                    }
                    
                    self.time_b32 = start;
                    self.tick = 0;
                }
            }
        }
    }

    pub fn preview_note(&mut self, pitch: Pitch) {
        self.state = PlayState::Preview;
        self.active_notes.clear();
        
        // Use the current instrument ID from the track editor
        // This will have been set via set_current_instrument_id before calling preview_note
        let instrument_id = self.current_instrument_id;
        
        self.active_notes.push(NoteEnvelope {
            note: Note {
                pitch,
                onset_b32: 0,
                duration_b32: 16,
                instrument_id,
            },
            start_tick: self.tick,
            end_tick: None,
        });
        self.preview_start = Some(Instant::now());
    }
    
    /// Set the current instrument ID for previewing notes
    pub fn set_current_instrument_id(&mut self, instrument_id: u32) {
        self.current_instrument_id = instrument_id;
    }

    pub fn clear_preview(&mut self) {
        if self.state == PlayState::Preview {
            // Mark all notes for release instead of immediately clearing
            let current_tick = self.tick;
            for note in &mut self.active_notes {
                if note.end_tick.is_none() {
                    note.end_tick = Some(current_tick);
                }
            }
            
            self.state = PlayState::Stopped;
            self.preview_start = None;
        }
    }
}

impl Iterator for Player {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if preview should end
        if let Some(start_time) = self.preview_start {
            const PREVIEW_DURATION_MS: u128 = 250;
            if start_time.elapsed().as_millis() > PREVIEW_DURATION_MS {
                self.clear_preview();
            }
        }

        match self.state {
            PlayState::Playing => {
                if self.tick % self.ticks_per_b32 == 0 {
                    if self.score.lock().unwrap().time_within_song(self.time_b32) {
                        self.update_active_notes();
                        self.handle_time_update();
                    } else {
                        // Mark all notes for release
                        let current_tick = self.tick;
                        for note in &mut self.active_notes {
                            if note.end_tick.is_none() {
                                note.end_tick = Some(current_tick);
                            }
                        }
                        self.stop();
                    }
                }
                self.tick += 1;
            }
            PlayState::Preview => {
                // Just continue playing the preview note
                self.tick += 1;
            }
            _ => return Some(0.0),
        }

        if self.active_notes.is_empty() {
            return Some(0.0);
        }

        let mut total_amplitudes: f64 = 0.0;
        let mut active_count = 0;

        for note_env in &self.active_notes {
            // Calculate envelope value for this note
            let envelope_value = self.calculate_envelope(note_env);
            
            // Skip notes with zero amplitude
            if envelope_value <= 0.0 {
                continue;
            }
            
            active_count += 1;
            
            // Get the instrument for this note
            if let Some(instrument) = self.instrument_registry.get_instrument(note_env.note.instrument_id) {
                // Generate sample using the instrument and apply envelope
                total_amplitudes += instrument.generate_sample(
                    note_env.note.pitch, 
                    self.tick, 
                    self.sample_rate
                ) * envelope_value;
            } else {
                // Fallback to basic sine wave if instrument not found
                let frequency = note_env.note.pitch.frequency(note_env.note.pitch.octave);
                total_amplitudes +=
                    (2.0 * PI * frequency * (self.tick as f64) / self.sample_rate as f64).sin() * envelope_value;
            }
        }

        // Normalize the output to prevent clipping
        let output = if active_count > 0 {
            total_amplitudes / active_count as f64
        } else {
            0.0
        };
        
        // Apply a simple limiter to prevent clipping (keep values between -1.0 and 1.0)
        let limited_output = output.max(-0.95).min(0.95);
        
        Some(limited_output)
    }
}
