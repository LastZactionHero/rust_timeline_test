// pitch.rs

use std::cmp::Ordering;
use std::fmt;
pub static OCTAVE_MAX: u16 = 8;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Tone {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

impl Tone {
    pub fn from_index(index: u16) -> Tone {
        match index {
            0 => Tone::C,
            1 => Tone::Cs,
            2 => Tone::D,
            3 => Tone::Ds,
            4 => Tone::E,
            5 => Tone::F,
            6 => Tone::Fs,
            7 => Tone::G,
            8 => Tone::Gs,
            9 => Tone::A,
            10 => Tone::As,
            11 => Tone::B,
            _ => panic!("Invalid tone index!"),
        }
    }
    pub fn index(&self) -> u16 {
        match self {
            Tone::C => 0,
            Tone::Cs => 1,
            Tone::D => 2,
            Tone::Ds => 3,
            Tone::E => 4,
            Tone::F => 5,
            Tone::Fs => 6,
            Tone::G => 7,
            Tone::Gs => 8,
            Tone::A => 9,
            Tone::As => 10,
            Tone::B => 11,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Tone::C => "C",
            Tone::Cs => "C#",
            Tone::D => "D",
            Tone::Ds => "D#",
            Tone::E => "E",
            Tone::F => "F",
            Tone::Fs => "F#",
            Tone::G => "G",
            Tone::Gs => "G#",
            Tone::A => "A",
            Tone::As => "A#",
            Tone::B => "B",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Pitch {
    pub tone: Tone,
    pub octave: u16,
}

impl Pitch {
    pub fn new(tone: Tone, octave: u16) -> Pitch {
        Pitch { tone, octave }
    }

    pub fn all() -> Vec<Pitch> {
        let mut pitches = vec![];
        for octave in 0..=OCTAVE_MAX {
            pitches.push(Pitch::new(Tone::C, octave));
            pitches.push(Pitch::new(Tone::Cs, octave));
            pitches.push(Pitch::new(Tone::D, octave));
            pitches.push(Pitch::new(Tone::Ds, octave));
            pitches.push(Pitch::new(Tone::E, octave));
            pitches.push(Pitch::new(Tone::F, octave));
            pitches.push(Pitch::new(Tone::Fs, octave));
            pitches.push(Pitch::new(Tone::G, octave));
            pitches.push(Pitch::new(Tone::Gs, octave));
            pitches.push(Pitch::new(Tone::A, octave));
            pitches.push(Pitch::new(Tone::As, octave));
            pitches.push(Pitch::new(Tone::B, octave));
        }
        pitches
    }

    pub fn next(&self) -> Option<Pitch> {
        if self.tone == Tone::B && self.octave == OCTAVE_MAX {
            return None;
        }
        if self.tone == Tone::B {
            return Some(Pitch::new(Tone::C, self.octave + 1));
        }
        Some(Pitch::new(
            Tone::from_index(self.tone.index() + 1),
            self.octave,
        ))
    }

    pub fn prev(&self) -> Option<Pitch> {
        if self.tone == Tone::C && self.octave == 0 {
            return None;
        }
        if self.tone == Tone::C {
            return Some(Pitch::new(Tone::B, self.octave - 1));
        }
        Some(Pitch::new(
            Tone::from_index(self.tone.index() - 1),
            self.octave,
        ))
    }

    pub fn frequency(&self, octave: u16) -> f64 {
        // Calculate the number of half steps from A4 (440 Hz)
        let half_steps_from_a4 = (octave as i32 - 4) * 12 + self.tone.index() as i32 - 9;

        // Calculate the frequency using the formula: 440 * 2^(n/12)
        440.0 * 2_f64.powf(half_steps_from_a4 as f64 / 12.0)
    }

    pub fn as_str(&self) -> String {
        format!("{}{}", self.tone.as_str(), self.octave)
    }
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.tone.as_str(), self.octave)
    }
}

impl PartialOrd for Pitch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.octave < other.octave {
            return Some(Ordering::Less);
        } else if self.octave > other.octave {
            return Some(Ordering::Greater);
        } else if self.tone.index() < other.tone.index() {
            return Some(Ordering::Less);
        } else if self.tone.index() > other.tone.index() {
            return Some(Ordering::Greater);
        }
        Some(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn test_tone_from_index() {
        assert_eq!(Tone::from_index(0), Tone::C);
        assert_eq!(Tone::from_index(1), Tone::Cs);
        assert_eq!(Tone::from_index(2), Tone::D);
        assert_eq!(Tone::from_index(11), Tone::B);
    }

    #[test]
    #[should_panic(expected = "Invalid tone index")]
    fn test_tone_from_index_invalid() {
        Tone::from_index(12); // Should panic
    }

    #[test]
    fn test_tone_index() {
        assert_eq!(Tone::C.index(), 0);
        assert_eq!(Tone::Cs.index(), 1);
        assert_eq!(Tone::D.index(), 2);
        assert_eq!(Tone::B.index(), 11);
    }

    #[test]
    fn test_tone_as_str() {
        assert_eq!(Tone::C.as_str(), "C");
        assert_eq!(Tone::Cs.as_str(), "C#");
        assert_eq!(Tone::D.as_str(), "D");
        assert_eq!(Tone::B.as_str(), "B");
    }

    #[test]
    fn test_pitch_new() {
        let p = Pitch::new(Tone::C, 4);
        assert_eq!(p.tone, Tone::C);
        assert_eq!(p.octave, 4);

        let p2 = Pitch::new(Tone::A, 5);
        assert_eq!(p2.tone, Tone::A);
        assert_eq!(p2.octave, 5);
    }

    #[test]
    fn test_pitch_next() {
        // Same octave
        let p1 = Pitch::new(Tone::C, 4);
        let p2 = p1.next().unwrap();
        assert_eq!(p2.tone, Tone::Cs);
        assert_eq!(p2.octave, 4);

        // Octave transition
        let p3 = Pitch::new(Tone::B, 4);
        let p4 = p3.next().unwrap();
        assert_eq!(p4.tone, Tone::C);
        assert_eq!(p4.octave, 5);

        // Upper limit
        let high_pitch = Pitch::new(Tone::B, OCTAVE_MAX);
        assert!(high_pitch.next().is_none());
    }

    #[test]
    fn test_pitch_prev() {
        // Same octave
        let p1 = Pitch::new(Tone::D, 4);
        let p2 = p1.prev().unwrap();
        assert_eq!(p2.tone, Tone::Cs);
        assert_eq!(p2.octave, 4);

        // Octave transition
        let p3 = Pitch::new(Tone::C, 4);
        let p4 = p3.prev().unwrap();
        assert_eq!(p4.tone, Tone::B);
        assert_eq!(p4.octave, 3);

        // Lower limit
        let low_pitch = Pitch::new(Tone::C, 0);
        assert!(low_pitch.prev().is_none());
    }

    #[test]
    fn test_pitch_frequency() {
        // A4 = 440Hz standard
        let p = Pitch::new(Tone::A, 4);
        assert!((p.frequency(4) - 440.0).abs() < 0.01);

        // C4 (middle C) should be around 261.63 Hz
        let middle_c = Pitch::new(Tone::C, 4);
        assert!((middle_c.frequency(4) - 261.63).abs() < 0.01);

        // Octave higher should double the frequency
        let c5 = Pitch::new(Tone::C, 5);
        assert!((c5.frequency(5) - 523.25).abs() < 0.01);
    }

    #[test]
    fn test_pitch_as_str() {
        assert_eq!(Pitch::new(Tone::C, 4).as_str(), "C4");
        assert_eq!(Pitch::new(Tone::Fs, 5).as_str(), "F#5");
        assert_eq!(Pitch::new(Tone::B, 3).as_str(), "B3");
    }

    #[test]
    fn test_pitch_all() {
        let all_pitches = Pitch::all();
        assert!(!all_pitches.is_empty());
        
        // Check expected length (OCTAVE_MAX + 1 octaves * 12 tones)
        assert_eq!(all_pitches.len(), ((OCTAVE_MAX + 1) * 12) as usize);
        
        // Check some expected pitches in the sequence
        assert!(all_pitches.contains(&Pitch::new(Tone::C, 1)));
        assert!(all_pitches.contains(&Pitch::new(Tone::G, 4)));
        assert!(all_pitches.contains(&Pitch::new(Tone::B, 7)));
    }

    #[test]
    fn test_pitch_comparison() {
        let p1 = Pitch::new(Tone::C, 4);
        let p2 = Pitch::new(Tone::D, 4);
        let p3 = Pitch::new(Tone::C, 5);
        
        // Same octave, different tones
        assert_eq!(p1.partial_cmp(&p2), Some(Ordering::Less));
        assert_eq!(p2.partial_cmp(&p1), Some(Ordering::Greater));
        
        // Different octaves
        assert_eq!(p1.partial_cmp(&p3), Some(Ordering::Less));
        assert_eq!(p3.partial_cmp(&p1), Some(Ordering::Greater));
        
        // Same pitch
        let p4 = Pitch::new(Tone::C, 4);
        assert_eq!(p1.partial_cmp(&p4), Some(Ordering::Equal));
    }
    
    #[test]
    fn test_pitch_display() {
        let p = Pitch::new(Tone::C, 4);
        assert_eq!(format!("{}", p), "C4");
        
        let p2 = Pitch::new(Tone::Fs, 5);
        assert_eq!(format!("{}", p2), "F#5");
    }
}
