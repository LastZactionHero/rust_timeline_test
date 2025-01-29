use std::str::FromStr;

use super::DrawComponent;
use crate::draw_components::Position;
use crate::mode::Mode;

pub struct StatusBarComponent {
    mode: Mode,
}

impl DrawComponent for StatusBarComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) {
        self.wb_string(buffer, pos, 0, 0, "|".repeat(pos.w));
        let mode_str = match self.mode {
            Mode::Normal => "[NOR]",
            Mode::Insert => "[INS]",
            Mode::Select => "[SEL]",
        };
        self.wb_string(buffer, pos, 0, 0, String::from_str(mode_str).unwrap());
    }
}

impl StatusBarComponent {
    pub fn new(mode: Mode) -> StatusBarComponent {
        StatusBarComponent { mode }
    }
}
