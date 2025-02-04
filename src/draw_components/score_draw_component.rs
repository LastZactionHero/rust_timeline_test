use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};

use super::{DrawComponent, DrawResult, ViewportDrawResult};
use crate::cursor::Cursor;
use crate::draw_components::Position;
use crate::events::InputEvent;
use crate::pitch::Pitch;
use crate::player::{PlayState, Player};
use crate::resolution::Resolution;
use crate::score::{Note, Score};

pub struct ScoreDrawComponent {
    score: Arc<Mutex<Score>>,
    play_state: PlayState,
    score_viewport: ScoreViewport,
    event_tx: mpsc::Sender<InputEvent>,
    cursor: Cursor,
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
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) -> Vec<DrawResult> {
        self.draw_pitches(buffer, pos);
        let viewport_draw_result = self.draw_score(
            buffer,
            &Position {
                x: pos.x + 4,
                y: pos.y,
                w: pos.w - 4,
                h: pos.h,
            },
        );
        vec![DrawResult::ViewportDrawResult(viewport_draw_result)]
    }
}

impl ScoreDrawComponent {
    pub fn new(
        score: Arc<Mutex<Score>>,
        play_state: PlayState,
        score_viewport: ScoreViewport,
        tx: mpsc::Sender<InputEvent>,
        cursor: Cursor,
    ) -> ScoreDrawComponent {
        ScoreDrawComponent {
            score,
            play_state,
            score_viewport,
            event_tx: tx,
            cursor,
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

    fn draw_score(&self, buffer: &mut Vec<Vec<char>>, pos: &super::Position) -> ViewportDrawResult {
        let pitches = self.visible_pitches(pos);

        for col in 0..pos.w - 1 {
            let bar_col = col % (self.score_viewport.resolution.bar_length_in_beats()) == 0;
            for (row, _pitch) in pitches.iter().enumerate() {
                let draw_char = if bar_col { '⎸' } else { '.' };
                self.wb(buffer, pos, col, row, draw_char);
            }

            if bar_col {
                let time_point_at_col = self.score_viewport.time_point
                    + (col as u64) * self.score_viewport.resolution.duration_b32();
                self.wb_string(
                    buffer,
                    pos,
                    col,
                    pitches.len(),
                    (time_point_at_col / (32)).to_string(),
                );
            }
        }

        let mut time_point = self.score_viewport.time_point;
        for col in 0..pos.w - 1 {
            for _ in 0..self.score_viewport.resolution.duration_b32() {
                for (row, _pitch) in pitches.iter().enumerate() {
                    if time_point == self.score_viewport.playback_time_point {
                        self.wb(buffer, pos, col, row, '░');
                    }
                }
                time_point += 1;
            }
        }
        let mut time_point = self.score_viewport.time_point;
        for col in 0..pos.w - 1 {
            for _ in 0..self.score_viewport.resolution.duration_b32() {
                let active_notes: HashMap<Pitch, Note> = self
                    .score
                    .lock()
                    .unwrap()
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

                    if self.cursor.visible() && self.cursor.equals(*pitch, time_point) {
                        self.wb(buffer, pos, col, row, 'C');
                    }
                }

                time_point += 1;
            }
        }

        ViewportDrawResult {
            pitch_low: *pitches.last().unwrap(),
            pitch_high: *pitches.first().unwrap(),
            time_point_start: self.score_viewport.time_point,
            time_point_end: time_point,
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
