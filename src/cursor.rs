use crate::pitch::Pitch;

#[derive(Clone, Copy)]
pub struct Cursor {
    pitch: Pitch,
    time_point: u64,
    visibility: Visibility,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Visibility {
    Hidden,
    Visible,
}

impl Cursor {
    pub fn new(pitch: Pitch, time_point: u64) -> Cursor {
        Cursor {
            pitch,
            time_point,
            visibility: Visibility::Hidden,
        }
    }

    pub fn resolution_align(self, duration: u64) -> Cursor {
        let mut next_cursor = self;
        next_cursor.time_point = next_cursor.time_point - next_cursor.time_point % duration;
        next_cursor
    }

    pub fn left(self, duration: u64) -> Cursor {
        let mut next_cursor = self;
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

    pub fn equals(self, pitch: Pitch, time_point: u64) -> bool {
        self.pitch == pitch && self.time_point == time_point
    }
}
