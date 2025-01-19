use std::fmt::Display;

use super::DrawComponent;
use crate::pitch::{self, Pitch, Tone};
use crate::score::Score;

pub struct ScoreDrawComponent {
    score: &'static Score,
}

#[derive(Clone, Copy)]
pub enum Resolution {
    Time1_4,
    Time1_8,
    Time1_16,
    Time1_32,
}

impl Resolution {
    pub fn as_str(&self) -> &str {
        match self {
            Resolution::Time1_4 => "1/4",
            Resolution::Time1_8 => "1/8",
            Resolution::Time1_16 => "1/16",
            Resolution::Time1_32 => "1/32",
        }
    }

    pub fn bar_length_in_beats(&self) -> u16 {
        match self {
            Resolution::Time1_4 => 4,
            Resolution::Time1_8 => 8,
            Resolution::Time1_16 => 16,
            Resolution::Time1_32 => 32,
        }
    }

    pub fn duration_b32(&self) -> u64 {
        match self {
            Resolution::Time1_4 => 8,
            Resolution::Time1_8 => 4,
            Resolution::Time1_16 => 2,
            Resolution::Time1_32 => 1,
        }
    }

    pub fn next_down(&self) -> Resolution {
        match self {
            Resolution::Time1_32 => Resolution::Time1_16,
            Resolution::Time1_16 => Resolution::Time1_8,
            Resolution::Time1_8 => Resolution::Time1_4,
            Resolution::Time1_4 => Resolution::Time1_4,
        }
    }

    pub fn next_up(&self) -> Resolution {
        match self {
            Resolution::Time1_4 => Resolution::Time1_8,
            Resolution::Time1_8 => Resolution::Time1_16,
            Resolution::Time1_16 => Resolution::Time1_32,
            Resolution::Time1_32 => Resolution::Time1_32,
        }
    }
}

pub struct ScoreViewport {
    middle_pitch: Pitch,
    resolution: Resolution,
    time_point: u64,
}

impl ScoreViewport {
    pub fn new(middle_pitch: Pitch, resolution: Resolution, time_point: u64) -> ScoreViewport {
        ScoreViewport {
            middle_pitch,
            resolution,
            time_point,
        }
    }
}

impl ScoreDrawComponent {
    pub fn new(score: &'static Score) -> ScoreDrawComponent {
        ScoreDrawComponent { score }
    }
}
impl DrawComponent for ScoreDrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) {
        self.draw_pitches(buffer, pos);
    }
}

impl ScoreDrawComponent {
    fn draw_pitches(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) {
        let num_pitches_to_display = pos.h - 1;

        let middle_pitch = Pitch::new(Tone::C, 4);
        let mut pitches = vec![middle_pitch];
        for _ in 0..(num_pitches_to_display / 2) {
            if let Some(prev_pitch) = pitches.last().unwrap().prev() {
                pitches.push(prev_pitch);
            }
        }
        pitches.reverse();
        for _ in 0..(num_pitches_to_display / 2) - 1 {
            if let Some(next_pitch) = pitches.last().unwrap().next() {
                pitches.push(next_pitch);
            }
        }

        for (i, pitch) in pitches.iter().enumerate() {
            self.wb_string(buffer, pos, 0, i, pitch.as_str());
        }
    }
}
