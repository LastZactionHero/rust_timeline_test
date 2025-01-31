use crate::pitch::Pitch;

pub struct Cursor {
    pitch: Pitch,
    time_point: u64,
    visibility: Visibility,
}

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

    pub fn left(&mut self) {
        if self.time_point > 0 {
            self.time_point -= 1;
        }
    }

    pub fn right(&mut self) {
        self.time_point += 1;
    }

    pub fn up(&mut self) {
        let next_pitch = self.pitch.next();
        if next_pitch.is_some() {
            self.pitch = next_pitch.unwrap();
        }
    }

    pub fn down(&mut self) {
        let prev_pitch = self.pitch.prev();
        if prev_pitch.is_some() {
            self.pitch = prev_pitch.unwrap();
        }
    }

    pub fn show(&mut self) {
        self.visibility = Visibility::Visible;
    }

    pub fn hide(&mut self) {
        self.visibility = Visibility::Hidden;
    }
}
