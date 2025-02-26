use crate::cursor::Cursor;
use crate::events::InputEvent;
use crate::loop_state::{LoopMode, LoopState};
use crate::pitch::Pitch;
use crate::player::PlayState;
use crate::score::{ActiveNote, NoteState, Score};
use crate::score_viewport::ScoreViewport;
use crate::selection_buffer::SelectionBuffer;
use std::sync::{mpsc, Arc, Mutex};

pub mod score_draw_component; // Keep this for backward compatibility for now
pub mod status_bar_component;
pub mod track_draw_component; // New module for track editing

#[derive(Clone, Copy)]
pub struct ViewportDrawResult {
    pub pitch_low: Pitch,
    pub pitch_high: Pitch,
    pub time_point_start: u64, // Inclusive
    pub time_point_end: u64,   // Exclusive
}

#[derive(Clone, Copy)]
pub enum DrawResult {
    ViewportDrawResult(ViewportDrawResult),
}

pub trait DrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult>;

    fn wb(&self, buffer: &mut Vec<Vec<char>>, pos: &Position, x: usize, y: usize, value: char) {
        buffer[pos.y + y][pos.x + x] = value;
    }

    fn wb_string(
        &self,
        buffer: &mut Vec<Vec<char>>,
        pos: &Position,
        x: usize,
        y: usize,
        value: String,
    ) {
        for (i, char) in value.chars().enumerate() {
            if pos.x + x + i >= buffer[pos.y].len() {
                break;
            }
            buffer[pos.y + y][pos.x + x + i] = char;
        }
    }
}

// Base component for shared functionality between ScoreDrawComponent and TrackDrawComponent
pub struct ScoreViewDrawComponent {
    pub score: Arc<Mutex<Score>>,
    pub play_state: PlayState,
    pub score_viewport: ScoreViewport,
    pub event_tx: mpsc::Sender<InputEvent>,
    pub cursor: Cursor,
    pub selection_buffer: SelectionBuffer,
    pub loop_state: LoopState,
}

impl DrawComponent for ScoreViewDrawComponent {
    fn draw(&self, _buffer: &mut Vec<Vec<char>>, _pos: &Position) -> Vec<DrawResult> {
        // This is just a base component - draw implementation is delegated to the concrete components
        vec![]
    }
}

impl ScoreViewDrawComponent {
    pub fn new(
        score: Arc<Mutex<Score>>,
        play_state: PlayState,
        score_viewport: ScoreViewport,
        event_tx: mpsc::Sender<InputEvent>,
        cursor: Cursor,
        selection_buffer: SelectionBuffer,
        loop_state: LoopState,
    ) -> Self {
        Self {
            score,
            play_state,
            score_viewport,
            event_tx,
            cursor,
            selection_buffer,
            loop_state,
        }
    }

    // Draw the playhead and loop markers
    pub fn draw_playhead(
        &self,
        buffer: &mut Vec<Vec<char>>,
        pos: &Position,
        num_rows: usize,
    ) -> u64 {
        let mut time_point = self.score_viewport.time_point;
        for col in 0..pos.w - 1 {
            for _ in 0..self.score_viewport.resolution.duration_b32() {
                for row in 0..num_rows {
                    if time_point == self.score_viewport.playback_time_point {
                        self.wb(buffer, pos, col, row, '░');
                    } else if self.loop_state.mode == LoopMode::Looping {
                        // Show loop start/end markers if loop mode is enabled
                        if let Some(start_time) = self.loop_state.start_time_b32 {
                            if time_point == start_time {
                                self.wb(buffer, pos, col, row, '░');
                            }
                        }
                        if let Some(end_time) = self.loop_state.end_time_b32 {
                            if time_point == end_time {
                                self.wb(buffer, pos, col, row, '░');
                            }
                        }
                    }
                }
                time_point += 1;
            }
        }
        time_point
    }
}

pub struct Position {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Position {
    fn right(&self) -> usize {
        self.x + self.w - 1
    }

    fn bottom(&self) -> usize {
        self.y + self.h - 1
    }
}

const BOX_TOP_LEFT: char = '╔';
const BOX_TOP_RIGHT: char = '╗';
const BOX_BOTTOM_LEFT: char = '╚';
const BOX_BOTTOM_RIGHT: char = '╝';
const BOX_HORIZONTAL: char = '═';
const BOX_VERTICAL: char = '║';
const BOX_RIGHT_DIVIDER: char = '╣';
const BOX_LEFT_DIVIDER: char = '╠';

pub struct Window {
    components: Vec<Box<dyn DrawComponent>>,
}

impl Window {
    pub fn new(components: Vec<Box<dyn DrawComponent>>) -> Window {
        Window { components }
    }
}

impl DrawComponent for Window {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult> {
        let mut results = vec![];
        for component in &self.components {
            results.append(component.draw(buffer, &pos).as_mut());
        }
        return results;
    }
}

pub struct BoxDrawComponent {
    component: Box<dyn DrawComponent>,
}

impl BoxDrawComponent {
    pub fn new(component: Box<dyn DrawComponent>) -> BoxDrawComponent {
        BoxDrawComponent { component }
    }
}

impl DrawComponent for BoxDrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult> {
        for x in pos.x + 1..pos.right() {
            self.wb(buffer, pos, x, 0, BOX_HORIZONTAL);
            self.wb(buffer, pos, x, pos.h - 1, BOX_HORIZONTAL);
        }
        for y in pos.y + 1..pos.bottom() {
            buffer[y][0] = BOX_VERTICAL;
            buffer[y][pos.x + pos.w - 1] = BOX_VERTICAL;
        }
        self.wb(buffer, pos, pos.x, pos.y, BOX_TOP_LEFT);
        self.wb(buffer, pos, pos.right(), pos.y, BOX_TOP_RIGHT);
        self.wb(buffer, pos, pos.x, pos.bottom(), BOX_BOTTOM_LEFT);
        self.wb(buffer, pos, pos.right(), pos.bottom(), BOX_BOTTOM_RIGHT);

        return self.component.draw(buffer, pos);
    }
}

pub struct VSplitDrawComponent {
    style: VSplitStyle,
    top_component: Box<dyn DrawComponent>,
    bottom_component: Box<dyn DrawComponent>,
}

#[derive(PartialEq, Eq)]
pub enum VSplitStyle {
    HalfWithDivider,
    StatusBarNoDivider,
}

impl VSplitDrawComponent {
    pub fn new(
        style: VSplitStyle,
        top_component: Box<dyn DrawComponent>,
        bottom_component: Box<dyn DrawComponent>,
    ) -> VSplitDrawComponent {
        VSplitDrawComponent {
            style,
            top_component,
            bottom_component,
        }
    }
}

impl DrawComponent for VSplitDrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult> {
        let pos_top = match self.style {
            VSplitStyle::HalfWithDivider => Position {
                x: pos.x + 1,
                y: pos.y + 1,
                w: pos.w - 2,
                h: pos.h / 2,
            },
            VSplitStyle::StatusBarNoDivider => Position {
                x: pos.x,
                y: pos.y,
                w: pos.w,
                h: pos.h - 1,
            },
        };

        let pos_bottom = match self.style {
            VSplitStyle::HalfWithDivider => Position {
                x: pos.x + 1,
                y: pos.y + pos.h / 2 + 2,
                w: pos.w - 2,
                h: pos.h / 2 - 2,
            },
            VSplitStyle::StatusBarNoDivider => Position {
                x: pos.x,
                y: pos.y + pos.h - 1,
                w: pos.w,
                h: 1,
            },
        };

        let mut result = vec![];
        result.append(self.top_component.draw(buffer, &pos_top).as_mut());
        result.append(self.bottom_component.draw(buffer, &pos_bottom).as_mut());

        if self.style == VSplitStyle::HalfWithDivider {
            for x in 1..pos.w - 1 {
                self.wb(buffer, pos, x, pos.h / 2 + 1, '═');
            }
            self.wb(buffer, pos, 0, pos.h / 2 + 1, BOX_LEFT_DIVIDER);
            self.wb(buffer, pos, pos.w - 1, pos.h / 2 + 1, BOX_RIGHT_DIVIDER);
        }

        result
    }
}

pub struct NullComponent {}

impl DrawComponent for NullComponent {
    fn draw(&self, _buffer: &mut Vec<Vec<char>>, _pos: &Position) -> Vec<DrawResult> {
        vec![]
    }
}

pub struct FillComponent {
    pub value: char,
}

impl DrawComponent for FillComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult> {
        for x in 0..pos.w {
            for y in 0..pos.h {
                self.wb(buffer, pos, x, y, self.value);
            }
        }
        vec![]
    }
}
