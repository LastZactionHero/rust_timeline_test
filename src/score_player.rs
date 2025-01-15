use crate::score::{Note, Score};

pub struct ScorePlayer {
    pub score: &'static Score,
    time_b32: u64,
}

impl ScorePlayer {
    pub fn create(score: &'static Score) -> ScorePlayer {
        ScorePlayer { score, time_b32: 0 }
    }

    pub fn reset(&mut self) {
        self.time_b32 = 0;
    }
}

impl Iterator for ScorePlayer {
    type Item = Vec<Note>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = None;
        if self.score.time_within_song(self.time_b32) {
            result = Some(self.score.active_notes_at_time(self.time_b32));
        }
        self.time_b32 += 1;
        result
    }
}
