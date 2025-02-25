use crate::draw_components::ViewportDrawResult;
use crate::pitch::Pitch;
use crate::resolution::Resolution;
use std::fmt;

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

    pub fn next_octave(&self) -> ScoreViewport {
        let mut new_viewport = *self;
        if let Some(next_pitch) = self.middle_pitch.next() {
            new_viewport.middle_pitch = next_pitch;
        }
        new_viewport
    }

    pub fn prev_octave(&self) -> ScoreViewport {
        let mut new_viewport = *self;
        if let Some(prev_pitch) = self.middle_pitch.prev() {
            new_viewport.middle_pitch = prev_pitch;
        }
        new_viewport
    }

    pub fn next_bar(&self, viewport_draw_result: &ViewportDrawResult) -> ScoreViewport {
        let mut new_viewport = *self;

        let is_more_than_playhead_halfway_through_viewport = self.playback_time_point
            > (viewport_draw_result.time_point_end - viewport_draw_result.time_point_start) / 2 + viewport_draw_result.time_point_start;
        if is_more_than_playhead_halfway_through_viewport {
            new_viewport.time_point += 32;
        }

        new_viewport
    }

    pub fn prev_bar(&self, viewport_draw_result: &ViewportDrawResult) -> ScoreViewport {
        let mut new_viewport = *self;

        let is_less_than_playhead_halfway_through_viewport = self.playback_time_point
            < (viewport_draw_result.time_point_end - viewport_draw_result.time_point_start) / 2 + viewport_draw_result.time_point_start;
        if is_less_than_playhead_halfway_through_viewport && self.time_point >= 32 {
            new_viewport.time_point -= 32;
        }

        new_viewport
    }

    pub fn increase_resolution(&self) -> ScoreViewport {
        let mut new_viewport = *self;
        new_viewport.resolution = self.resolution.next_up();
        new_viewport
    }

    pub fn decrease_resolution(&self) -> ScoreViewport {
        let mut new_viewport = *self;
        new_viewport.resolution = self.resolution.next_down();
        new_viewport
    }

    pub fn set_playback_time(&self, time: u64) -> ScoreViewport {
        let mut new_viewport = *self;
        new_viewport.playback_time_point = time;
        new_viewport
    }

    pub fn set_time_point(&self, time: u64) -> ScoreViewport {
        let mut new_viewport = *self;
        new_viewport.time_point = time;
        new_viewport
    }
}

impl fmt::Display for ScoreViewport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.middle_pitch, self.time_point, self.playback_time_point
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::{Pitch, Tone};

    fn create_test_viewport() -> ScoreViewport {
        ScoreViewport::new(
            Pitch::new(Tone::C, 4), // Middle C
            Resolution::Time1_16,   // 1/16 note resolution
            32,                     // Starting at beat 1 (in 32nd notes)
            0,                      // Playback at beginning
        )
    }

    #[test]
    fn test_viewport_new() {
        let viewport = create_test_viewport();
        assert_eq!(viewport.middle_pitch, Pitch::new(Tone::C, 4));
        assert_eq!(viewport.resolution, Resolution::Time1_16);
        assert_eq!(viewport.time_point, 32);
        assert_eq!(viewport.playback_time_point, 0);
    }

    #[test]
    fn test_next_octave() {
        let viewport = create_test_viewport();
        let next = viewport.next_octave();
        assert_eq!(next.middle_pitch, Pitch::new(Tone::Cs, 4));
        assert_eq!(next.time_point, viewport.time_point);
        assert_eq!(next.resolution, viewport.resolution);
    }

    #[test]
    fn test_prev_octave() {
        let viewport = create_test_viewport();
        let prev = viewport.prev_octave();
        assert_eq!(prev.middle_pitch, Pitch::new(Tone::B, 3));
        assert_eq!(prev.time_point, viewport.time_point);
        assert_eq!(prev.resolution, viewport.resolution);
    }

    #[test]
    fn test_next_bar() {
        let viewport = create_test_viewport();
        
        // Playhead in first half, should stay at same position
        let viewport_result = ViewportDrawResult {
            pitch_low: Pitch::new(Tone::C, 3),
            pitch_high: Pitch::new(Tone::C, 5),
            time_point_start: 32,
            time_point_end: 96,
        };
        
        // Playhead in first half
        let viewport1 = viewport.set_playback_time(40);
        let next1 = viewport1.next_bar(&viewport_result);
        assert_eq!(next1.time_point, 32); // Stays the same
        
        // Playhead in second half, should advance
        let viewport2 = viewport.set_playback_time(70);
        let next2 = viewport2.next_bar(&viewport_result);
        assert_eq!(next2.time_point, 64); // Moves forward by 32
    }

    #[test]
    fn test_prev_bar() {
        let viewport = create_test_viewport().set_time_point(64);
        
        // Create viewport result
        let viewport_result = ViewportDrawResult {
            pitch_low: Pitch::new(Tone::C, 3),
            pitch_high: Pitch::new(Tone::C, 5),
            time_point_start: 64,
            time_point_end: 128,
        };
        
        // Playhead in second half, should stay at same position
        let viewport1 = viewport.set_playback_time(100);
        let prev1 = viewport1.prev_bar(&viewport_result);
        assert_eq!(prev1.time_point, 64); // Stays the same
        
        // Playhead in first half, should move back
        let viewport2 = viewport.set_playback_time(70);
        let prev2 = viewport2.prev_bar(&viewport_result);
        assert_eq!(prev2.time_point, 32); // Moves back by 32
        
        // At beginning, shouldn't go negative
        let viewport3 = ScoreViewport::new(
            Pitch::new(Tone::C, 4),
            Resolution::Time1_16,
            0,
            0
        );
        let viewport_result3 = ViewportDrawResult {
            pitch_low: Pitch::new(Tone::C, 3),
            pitch_high: Pitch::new(Tone::C, 5),
            time_point_start: 0,
            time_point_end: 64,
        };
        let prev3 = viewport3.prev_bar(&viewport_result3);
        assert_eq!(prev3.time_point, 0); // Stays at 0
    }

    #[test]
    fn test_increase_resolution() {
        let viewport = create_test_viewport();
        assert_eq!(viewport.resolution, Resolution::Time1_16);
        
        let increased = viewport.increase_resolution();
        assert_eq!(increased.resolution, Resolution::Time1_32);
        
        // At max resolution, stays the same
        let max = increased.increase_resolution();
        assert_eq!(max.resolution, Resolution::Time1_32);
    }

    #[test]
    fn test_decrease_resolution() {
        let viewport = create_test_viewport();
        assert_eq!(viewport.resolution, Resolution::Time1_16);
        
        let decreased = viewport.decrease_resolution();
        assert_eq!(decreased.resolution, Resolution::Time1_8);
        
        let decreased2 = decreased.decrease_resolution();
        assert_eq!(decreased2.resolution, Resolution::Time1_4);
        
        // At min resolution, stays the same
        let min = decreased2.decrease_resolution();
        assert_eq!(min.resolution, Resolution::Time1_4);
    }

    #[test]
    fn test_set_playback_time() {
        let viewport = create_test_viewport();
        assert_eq!(viewport.playback_time_point, 0);
        
        let updated = viewport.set_playback_time(64);
        assert_eq!(updated.playback_time_point, 64);
        assert_eq!(updated.time_point, viewport.time_point); // Time point unchanged
    }

    #[test]
    fn test_set_time_point() {
        let viewport = create_test_viewport();
        assert_eq!(viewport.time_point, 32);
        
        let updated = viewport.set_time_point(64);
        assert_eq!(updated.time_point, 64);
        assert_eq!(updated.playback_time_point, viewport.playback_time_point); // Playback time unchanged
    }

    #[test]
    fn test_display() {
        let viewport = create_test_viewport();
        assert_eq!(format!("{}", viewport), "C4 32 0");
        
        let viewport2 = ScoreViewport::new(
            Pitch::new(Tone::F, 5),
            Resolution::Time1_8,
            64,
            96
        );
        assert_eq!(format!("{}", viewport2), "F5 64 96");
    }
}
