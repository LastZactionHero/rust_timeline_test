use std::f64::consts::PI;
use std::sync::Arc;
use crate::pitch::Pitch;

pub trait Instrument: Send + Sync {
    /// Generate a sample for the given pitch at the given time point (in sample ticks)
    fn generate_sample(&self, pitch: Pitch, time_ticks: u64, sample_rate: u64) -> f64;
    
    /// Clone the instrument (needed because trait objects can't be cloned directly)
    fn clone_instrument(&self) -> Box<dyn Instrument>;
    
    /// Get the name of the instrument
    fn name(&self) -> &str;
    
    /// Get the instrument ID
    fn id(&self) -> u32;
}

/// Simple sine wave instrument
pub struct SineInstrument {
    id: u32,
    name: String,
    amplitude: f64,
}

impl SineInstrument {
    pub fn new(id: u32, name: String, amplitude: f64) -> Self {
        Self { id, name, amplitude }
    }
}

impl Instrument for SineInstrument {
    fn generate_sample(&self, pitch: Pitch, time_ticks: u64, sample_rate: u64) -> f64 {
        let frequency = pitch.frequency(pitch.octave);
        self.amplitude * (2.0 * PI * frequency * (time_ticks as f64) / (sample_rate as f64)).sin()
    }
    
    fn clone_instrument(&self) -> Box<dyn Instrument> {
        Box::new(SineInstrument {
            id: self.id,
            name: self.name.clone(),
            amplitude: self.amplitude,
        })
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn id(&self) -> u32 {
        self.id
    }
}

/// FM Synthesizer instrument with modulator and carrier frequencies
pub struct FMSynthesizer {
    id: u32,
    name: String,
    carrier_amplitude: f64,
    modulator_amplitude: f64,
    modulator_ratio: f64,
    feedback: f64,
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
}

impl FMSynthesizer {
    pub fn new(
        id: u32, 
        name: String,
        carrier_amplitude: f64,
        modulator_amplitude: f64,
        modulator_ratio: f64,
        feedback: f64,
        attack: f64,
        decay: f64,
        sustain: f64,
        release: f64,
    ) -> Self {
        Self {
            id,
            name,
            carrier_amplitude,
            modulator_amplitude,
            modulator_ratio,
            feedback,
            attack,
            decay,
            sustain,
            release,
        }
    }
    
    /// Create a bass FM instrument preset
    pub fn bass(id: u32) -> Self {
        Self::new(
            id,
            "FM Bass".to_string(),
            0.8,     // carrier_amplitude
            3.0,     // modulator_amplitude (higher = more harmonics)
            1.0,     // modulator_ratio (1:1 ratio)
            0.1,     // feedback
            0.01,    // fast attack
            0.2,     // medium decay
            0.4,     // moderate sustain
            0.3,     // medium release
        )
    }
    
    /// Create a bell-like FM instrument preset
    pub fn bell(id: u32) -> Self {
        Self::new(
            id,
            "FM Bell".to_string(),
            0.6,     // carrier_amplitude
            2.0,     // modulator_amplitude
            2.0,     // modulator_ratio (2:1 ratio - creates bell-like harmonics)
            0.0,     // no feedback
            0.01,    // fast attack
            0.8,     // long decay
            0.1,     // low sustain
            1.0,     // long release
        )
    }
    
    /// Create a brass-like FM instrument preset
    pub fn brass(id: u32) -> Self {
        Self::new(
            id,
            "FM Brass".to_string(),
            0.7,     // carrier_amplitude
            4.0,     // modulator_amplitude (higher for brass timbres)
            1.0,     // modulator_ratio (1:1)
            0.2,     // moderate feedback
            0.05,    // medium attack
            0.1,     // short decay
            0.7,     // high sustain
            0.2,     // medium release
        )
    }
    
    /// Create a plucked string-like FM instrument preset
    pub fn pluck(id: u32) -> Self {
        Self::new(
            id,
            "FM Pluck".to_string(),
            0.7,     // carrier_amplitude
            5.0,     // modulator_amplitude (high for initial pluck)
            0.5,     // modulator_ratio (0.5:1 ratio)
            0.1,     // some feedback
            0.001,   // very fast attack
            0.15,    // fast decay
            0.2,     // low sustain
            0.3,     // medium release
        )
    }
    
    // Calculate envelope value based on ADSR parameters and note progress (0.0 to 1.0)
    fn envelope(&self, progress: f64) -> f64 {
        if progress < self.attack {
            // Attack phase
            return progress / self.attack;
        } else if progress < self.attack + self.decay {
            // Decay phase
            let decay_progress = (progress - self.attack) / self.decay;
            return 1.0 - (1.0 - self.sustain) * decay_progress;
        } else if progress < 1.0 - self.release {
            // Sustain phase
            return self.sustain;
        } else {
            // Release phase
            let release_progress = (progress - (1.0 - self.release)) / self.release;
            return self.sustain * (1.0 - release_progress);
        }
    }
}

impl Instrument for FMSynthesizer {
    fn generate_sample(&self, pitch: Pitch, time_ticks: u64, sample_rate: u64) -> f64 {
        let carrier_freq = pitch.frequency(pitch.octave);
        let modulator_freq = carrier_freq * self.modulator_ratio;
        
        // Assume duration for envelope
        let note_duration_ticks = sample_rate; // 1 second, simplified
        let progress = (time_ticks % note_duration_ticks) as f64 / note_duration_ticks as f64;
        
        // Apply envelope to modulator amplitude
        let env_value = self.envelope(progress);
        let mod_amplitude = self.modulator_amplitude * env_value;
        
        // Calculate modulator signal with feedback
        let mod_phase = 2.0 * PI * modulator_freq * (time_ticks as f64) / (sample_rate as f64);
        let feedback_amount = self.feedback * (2.0 * PI * carrier_freq * ((time_ticks - 1) as f64) / (sample_rate as f64)).sin();
        let mod_signal = (mod_phase + feedback_amount).sin() * mod_amplitude;
        
        // Modulate the carrier frequency
        let carrier_phase = 2.0 * PI * carrier_freq * (time_ticks as f64) / (sample_rate as f64);
        let carrier_signal = (carrier_phase + mod_signal).sin();
        
        // Apply envelope to final output
        carrier_signal * self.carrier_amplitude * env_value
    }
    
    fn clone_instrument(&self) -> Box<dyn Instrument> {
        Box::new(FMSynthesizer {
            id: self.id,
            name: self.name.clone(),
            carrier_amplitude: self.carrier_amplitude,
            modulator_amplitude: self.modulator_amplitude,
            modulator_ratio: self.modulator_ratio,
            feedback: self.feedback,
            attack: self.attack,
            decay: self.decay,
            sustain: self.sustain,
            release: self.release,
        })
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn id(&self) -> u32 {
        self.id
    }
}

/// Create a default set of instruments
pub fn create_default_instruments() -> Vec<Box<dyn Instrument>> {
    vec![
        Box::new(FMSynthesizer::bass(0)),      // Instrument 0: Bass
        Box::new(FMSynthesizer::brass(1)),     // Instrument 1: Brass
        Box::new(FMSynthesizer::bell(2)),      // Instrument 2: Bell
        Box::new(FMSynthesizer::pluck(3)),     // Instrument 3: Pluck
    ]
}

/// A registry of all available instruments
pub struct InstrumentRegistry {
    instruments: Vec<Box<dyn Instrument>>,
}

impl InstrumentRegistry {
    pub fn new() -> Self {
        Self { 
            instruments: create_default_instruments() 
        }
    }
    
    pub fn get_instrument(&self, id: u32) -> Option<&dyn Instrument> {
        self.instruments.iter()
            .find(|instrument| instrument.id() == id)
            .map(|instrument| instrument.as_ref())
    }
    
    pub fn get_all_instruments(&self) -> Vec<&dyn Instrument> {
        self.instruments.iter().map(|i| i.as_ref()).collect()
    }
}

impl Clone for InstrumentRegistry {
    fn clone(&self) -> Self {
        let cloned_instruments = self.instruments.iter()
            .map(|instrument| instrument.clone_instrument())
            .collect();
            
        Self {
            instruments: cloned_instruments,
        }
    }
}