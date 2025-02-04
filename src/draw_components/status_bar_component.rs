use std::str::FromStr;
use std::sync::{Arc, Mutex};

use super::score_draw_component::ScoreViewport;
use super::{DrawComponent, DrawResult};
use crate::cursor::Cursor;
use crate::draw_components::Position;
use crate::mode::Mode;

pub struct StatusBarComponent {
    mode: Arc<Mutex<Mode>>,
    cursor: Cursor,
    score_viewport: ScoreViewport,
}

impl DrawComponent for StatusBarComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult> {
        self.wb_string(buffer, pos, 0, 0, "|".repeat(pos.w));
        let mode_str = match *self.mode.lock().unwrap() {
            Mode::Normal => "[NOR]",
            Mode::Insert => "[INS]",
            Mode::Select => "[SEL]",
        };
        let status_str = format!(
            "{} [Cursor: {}] [Score Viewport: {}]",
            mode_str, self.cursor, self.score_viewport
        );
        self.wb_string(buffer, pos, 0, 0, status_str);
        vec![]
    }
}

impl StatusBarComponent {
    pub fn new(
        mode: Arc<Mutex<Mode>>,
        cursor: Cursor,
        score_viewport: ScoreViewport,
    ) -> StatusBarComponent {
        StatusBarComponent {
            mode,
            cursor,
            score_viewport,
        }
    }
}
