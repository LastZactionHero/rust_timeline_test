#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoopMode {
    Disabled,
    Looping,
}

#[derive(Debug, Clone, Copy)]
pub struct LoopState {
    pub start_time_b32: Option<u64>,
    pub end_time_b32: Option<u64>,
    pub mode: LoopMode,
}

impl LoopState {
    pub fn new() -> Self {
        Self {
            start_time_b32: None,
            end_time_b32: None,
            mode: LoopMode::Disabled,
        }
    }

    pub fn mark(&self, time_b32: u64) -> Self {
        let mut new_state = *self;
        match (new_state.start_time_b32, new_state.end_time_b32) {
            (None, _) => {
                // First mark
                new_state.start_time_b32 = Some(time_b32);
            }
            (Some(start), None) => {
                // Second mark
                if time_b32 < start {
                    new_state.end_time_b32 = new_state.start_time_b32;
                    new_state.start_time_b32 = Some(time_b32);
                } else {
                    new_state.end_time_b32 = Some(time_b32);
                }
            }
            (Some(_), Some(_)) => {
                // If both times are set, clear and start over
                new_state.start_time_b32 = Some(time_b32);
                new_state.end_time_b32 = None;
            }
        }
        new_state
    }

    pub fn set_mode(&self, mode: LoopMode) -> Self {
        let mut new_state = *self;
        new_state.mode = mode;
        new_state
    }

    pub fn toggle_mode(&self) -> Self {
        let mut new_state = *self;
        new_state.mode = match new_state.mode {
            LoopMode::Disabled => LoopMode::Looping,
            LoopMode::Looping => LoopMode::Disabled,
        };
        new_state
    }

    pub fn clear(&self) -> Self {
        Self::new()
    }

    pub fn is_looping(&self) -> bool {
        self.mode == LoopMode::Looping
            && self.start_time_b32.is_some()
            && self.end_time_b32.is_some()
    }
}

impl Default for LoopState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_state_new() {
        let state = LoopState::new();
        assert_eq!(state.start_time_b32, None);
        assert_eq!(state.end_time_b32, None);
        assert_eq!(state.mode, LoopMode::Disabled);
    }

    #[test]
    fn test_loop_state_default() {
        let state = LoopState::default();
        assert_eq!(state.start_time_b32, None);
        assert_eq!(state.end_time_b32, None);
        assert_eq!(state.mode, LoopMode::Disabled);
    }

    #[test]
    fn test_loop_state_mark_first() {
        let state = LoopState::new();
        let marked = state.mark(64);
        
        assert_eq!(marked.start_time_b32, Some(64));
        assert_eq!(marked.end_time_b32, None);
    }

    #[test]
    fn test_loop_state_mark_second_forward() {
        let state = LoopState::new().mark(64);
        let marked = state.mark(96);
        
        assert_eq!(marked.start_time_b32, Some(64));
        assert_eq!(marked.end_time_b32, Some(96));
    }

    #[test]
    fn test_loop_state_mark_second_backward() {
        let state = LoopState::new().mark(64);
        let marked = state.mark(32);
        
        // Should swap start and end times so start is always lower
        assert_eq!(marked.start_time_b32, Some(32));
        assert_eq!(marked.end_time_b32, Some(64));
    }

    #[test]
    fn test_loop_state_mark_reset() {
        let state = LoopState::new().mark(32).mark(64);
        assert_eq!(state.start_time_b32, Some(32));
        assert_eq!(state.end_time_b32, Some(64));
        
        // Third mark should reset loop
        let reset = state.mark(48);
        assert_eq!(reset.start_time_b32, Some(48));
        assert_eq!(reset.end_time_b32, None);
    }

    #[test]
    fn test_loop_state_set_mode() {
        let state = LoopState::new();
        assert_eq!(state.mode, LoopMode::Disabled);
        
        let looping = state.set_mode(LoopMode::Looping);
        assert_eq!(looping.mode, LoopMode::Looping);
    }

    #[test]
    fn test_loop_state_toggle_mode() {
        let state = LoopState::new();
        assert_eq!(state.mode, LoopMode::Disabled);
        
        let toggled1 = state.toggle_mode();
        assert_eq!(toggled1.mode, LoopMode::Looping);
        
        let toggled2 = toggled1.toggle_mode();
        assert_eq!(toggled2.mode, LoopMode::Disabled);
    }

    #[test]
    fn test_loop_state_clear() {
        let state = LoopState::new()
            .mark(32)
            .mark(64)
            .set_mode(LoopMode::Looping);
            
        assert_eq!(state.start_time_b32, Some(32));
        assert_eq!(state.end_time_b32, Some(64));
        assert_eq!(state.mode, LoopMode::Looping);
        
        let cleared = state.clear();
        assert_eq!(cleared.start_time_b32, None);
        assert_eq!(cleared.end_time_b32, None);
        assert_eq!(cleared.mode, LoopMode::Disabled);
    }

    #[test]
    fn test_loop_state_is_looping() {
        // Not looping - mode disabled
        let state1 = LoopState::new().mark(32).mark(64);
        assert!(!state1.is_looping());
        
        // Not looping - missing end time
        let state2 = LoopState::new()
            .mark(32)
            .set_mode(LoopMode::Looping);
        assert!(!state2.is_looping());
        
        // Not looping - missing both times
        let state3 = LoopState::new()
            .set_mode(LoopMode::Looping);
        assert!(!state3.is_looping());
        
        // Is looping - has both times and mode enabled
        let state4 = LoopState::new()
            .mark(32)
            .mark(64)
            .set_mode(LoopMode::Looping);
        assert!(state4.is_looping());
    }
}
