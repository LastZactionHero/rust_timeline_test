use std::fmt;

use crossterm::cursor;

use crate::pitch::Pitch;

#[derive(Clone, Copy)]
pub struct Cursor {
    pitch: Pitch,
    time_point: u64,
    visibility: Visibility,
    mode: CursorMode,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Visibility {
    Hidden,
    Visible,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CursorMode {
    Move,
    Insert(u64), // Start insert onset
    Select(Pitch, u64), // Start select onset and pitch
                 // SELECT
                 // CUT
                 // YANK
}

impl Cursor {
    pub fn new(pitch: Pitch, time_point: u64) -> Cursor {
        Cursor {
            pitch,
            time_point,
            visibility: Visibility::Hidden,
            mode: CursorMode::Move,
        }
    }

    pub fn resolution_align(self, duration: u64) -> Cursor {
        let mut next_cursor = self;
        next_cursor.time_point = next_cursor.time_point - next_cursor.time_point % duration;
        next_cursor
    }

    pub fn left(self, duration: u64) -> Cursor {
        let mut next_cursor = self;

        // Don't allow moving cursor before onset on insert.
        if let CursorMode::Insert(onset_b32) = self.mode {
            if self.time_point == onset_b32 {
                return next_cursor;
            }
        }

        if self.time_point >= duration {
            next_cursor.time_point -= duration;
        } else {
            next_cursor.time_point = 0;
        }
        next_cursor.time_point = next_cursor.time_point - next_cursor.time_point % duration;
        next_cursor
    }

    pub fn right(self, duration: u64) -> Cursor {
        let mut next_cursor = self;
        next_cursor.time_point += duration;
        next_cursor.time_point = next_cursor.time_point - next_cursor.time_point % duration;
        next_cursor
    }

    pub fn up(self) -> Cursor {
        let mut next_cursor = self;
        let next_pitch = self.pitch.next();
        if next_pitch.is_some() {
            next_cursor.pitch = next_pitch.unwrap();
        }
        next_cursor
    }

    pub fn down(self) -> Cursor {
        let mut next_cursor = self;
        let prev_pitch = self.pitch.prev();
        if prev_pitch.is_some() {
            next_cursor.pitch = prev_pitch.unwrap();
        }
        next_cursor
    }

    pub fn show(self) -> Cursor {
        let mut next_cursor = self;
        next_cursor.visibility = Visibility::Visible;
        next_cursor
    }

    pub fn hide(self) -> Cursor {
        let mut next_cursor = self;
        next_cursor.visibility = Visibility::Hidden;
        next_cursor
    }

    pub fn visible(self) -> bool {
        self.visibility == Visibility::Visible
    }

    pub fn visible_at(self, pitch: Pitch, time_point: u64) -> bool {
        if !self.visible() || self.pitch != pitch {
            return false;
        }
        match self.mode {
            CursorMode::Move => time_point == self.time_point,
            CursorMode::Insert(onset_b32) => {
                time_point >= onset_b32 && time_point <= self.time_point
            }
            CursorMode::Select(pitch, onset_b32) => false,
        }
    }

    pub fn time_point(self) -> u64 {
        self.time_point
    }

    pub fn pitch(self) -> Pitch {
        self.pitch
    }

    pub fn mode(self) -> CursorMode {
        self.mode
    }

    pub fn start_insert(self) -> Cursor {
        let mut cursor = self;
        cursor.mode = CursorMode::Insert(self.time_point);
        cursor
    }

    pub fn end_insert(self) -> Cursor {
        let mut cursor = self;
        cursor.mode = CursorMode::Move;
        cursor
    }

    pub fn start_select(self) -> Cursor {
        let mut cursor = self;
        cursor.mode = CursorMode::Select(self.pitch, self.time_point);
        cursor
    }

    pub fn end_select(self) -> Cursor {
        let mut cursor = self;
        cursor.mode = CursorMode::Move;
        cursor
    }

    pub fn cancel(self) -> Cursor {
        let mut cursor = self;
        cursor.mode = CursorMode::Move;
        cursor
    }
}

impl fmt::Display for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.time_point, self.pitch)
    }
}
