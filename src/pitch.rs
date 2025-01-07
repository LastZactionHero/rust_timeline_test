// pitch.rs

use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Pitch {
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

impl Pitch {
    pub fn as_str(&self) -> &str {
        match self {
            Pitch::C => "C",
            Pitch::Cs => "C#",
            Pitch::D => "D",
            Pitch::Ds => "D#",
            Pitch::E => "E",
            Pitch::F => "F",
            Pitch::Fs => "F#",
            Pitch::G => "G",
            Pitch::Gs => "G#",
            Pitch::A => "A",
            Pitch::As => "A#",
            Pitch::B => "B",
        }
    }

    pub fn row_index(&self) -> u16 {
        match self {
            Pitch::C => 0,
            Pitch::Cs => 1,
            Pitch::D => 2,
            Pitch::Ds => 3,
            Pitch::E => 4,
            Pitch::F => 5,
            Pitch::Fs => 6,
            Pitch::G => 7,
            Pitch::Gs => 8,
            Pitch::A => 9,
            Pitch::As => 10,
            Pitch::B => 11,
        }
    }

    pub fn from_row_index(row: u16) -> Pitch {
        match row {
            0 => Pitch::C,
            1 => Pitch::Cs,
            2 => Pitch::D,
            3 => Pitch::Ds,
            4 => Pitch::E,
            5 => Pitch::F,
            6 => Pitch::Fs,
            7 => Pitch::G,
            8 => Pitch::Gs,
            9 => Pitch::A,
            10 => Pitch::As,
            11 => Pitch::B,
            _ => panic!("Invalid row index for pitch: {}", row),
        }
    }
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
