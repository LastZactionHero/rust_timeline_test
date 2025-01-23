use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Index;
use std::sync::mpsc;

use super::DrawComponent;
use crate::draw_components::Position;
use crate::events::InputEvent;
use crate::pitch::{self, Pitch, Tone};
use crate::score::{Note, Score};

pub struct ScoreDrawComponent {
    score: &'static Score,
    score_viewport: ScoreViewport,
    event_tx: mpsc::Sender<InputEvent>,
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

    pub fn bar_length_in_beats(&self) -> usize {
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

#[derive(Clone, Copy)]
pub struct ScoreViewport {
    pub middle_pitch: Pitch,
    pub resolution: Resolution,
    pub time_point: u64,
    pub playback_time_point: u64,
}

impl ScoreViewport {
    pub fn new(
        middle_pitch: Pitch,
        resolution: Resolution,
        time_point: u64,
        playback_time_point: u64,
    ) -> ScoreViewport {
        ScoreViewport {
            middle_pitch,
            resolution,
            time_point,
            playback_time_point,
        }
    }
}

impl DrawComponent for ScoreDrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) {
        self.draw_pitches(buffer, pos);
        self.draw_score(
            buffer,
            &Position {
                x: pos.x + 4,
                y: pos.y,
                w: pos.w - 4,
                h: pos.h,
            },
        );
    }
}

impl ScoreDrawComponent {
    pub fn new(
        score: &'static Score,
        score_viewport: ScoreViewport,
        tx: mpsc::Sender<InputEvent>,
    ) -> ScoreDrawComponent {
        ScoreDrawComponent {
            score,
            score_viewport,
            event_tx: tx,
        }
    }

    fn visible_pitches(&self, pos: &Position) -> Vec<Pitch> {
        let num_pitches_to_display = pos.h - 1;

        let middle_pitch = self.score_viewport.middle_pitch;
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
        pitches.reverse();
        pitches
    }

    fn draw_score(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) {
        let pitches = self.visible_pitches(pos);

        for col in 0..pos.w - 1 {
            let bar_col = col % (self.score_viewport.resolution.bar_length_in_beats()) == 0;
            for (row, _pitch) in pitches.iter().enumerate() {
                let draw_char = if bar_col { '-' } else { '.' };
                self.wb(buffer, pos, col, row, draw_char);
            }
        }

        let mut time_point = self.score_viewport.time_point;
        for col in 0..pos.w - 1 {
            for _ in 0..self.score_viewport.resolution.duration_b32() {
                for (row, pitch) in pitches.iter().enumerate() {
                    if time_point == self.score_viewport.playback_time_point {
                        self.wb(buffer, pos, col, row, '░');
                    }
                }
                time_point += 1;
            }
        }
        let mut playhead_in_view = false;
        let mut time_point = self.score_viewport.time_point;
        for col in 0..pos.w - 1 {
            for _ in 0..self.score_viewport.resolution.duration_b32() {
                let active_notes: HashMap<Pitch, Note> = self
                    .score
                    .notes_starting_at_time(time_point)
                    .into_iter()
                    .map(|note| (note.pitch, note))
                    .collect();

                for (row, pitch) in pitches.iter().enumerate() {
                    if let Some(note) = active_notes.get(pitch) {
                        self.wb_string(
                            buffer,
                            pos,
                            col,
                            row,
                            self.note_string(note, time_point, &self.score_viewport.resolution),
                        );
                    }
                }
                if time_point == self.score_viewport.playback_time_point {
                    playhead_in_view = true;
                }
                time_point += 1;
            }
        }
        if !playhead_in_view {
            self.event_tx
                .send(InputEvent::PlayheadOutOfViewport)
                .unwrap();
        }
    }

    fn draw_pitches(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) {
        for (i, pitch) in self.visible_pitches(pos).iter().enumerate() {
            self.wb_string(buffer, pos, 0, i, pitch.as_str());
        }
    }

    fn note_string(&self, note: &Note, time_point: u64, resolution: &Resolution) -> String {
        let mut note_str = String::new();
        let note_len_chars = std::cmp::max(1, note.duration_b32 / resolution.duration_b32());
        for i in 0..note_len_chars {
            let note_char =
                if i == (note_len_chars - 1) && time_point == note.onset_b32 && note_len_chars == 1
                {
                    '▉'
                } else if i == 0 {
                    '█'
                } else {
                    '▒'
                };
            note_str.push(note_char);
        }
        note_str
    }
}
