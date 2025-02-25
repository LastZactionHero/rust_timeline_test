#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resolution {
    Time1_4,
    Time1_8,
    Time1_16,
    Time1_32,
}

impl Resolution {
    pub fn as_str(&self) -> &str {
        match self {
            Resolution::Time1_4 => "1/4",
            Resolution::Time1_8 => "1/8",
            Resolution::Time1_16 => "1/16",
            Resolution::Time1_32 => "1/32",
        }
    }

    pub fn bar_length_in_beats(&self) -> usize {
        match self {
            Resolution::Time1_4 => 4,
            Resolution::Time1_8 => 8,
            Resolution::Time1_16 => 16,
            Resolution::Time1_32 => 32,
        }
    }

    pub fn duration_b32(&self) -> u64 {
        match self {
            Resolution::Time1_4 => 8,
            Resolution::Time1_8 => 4,
            Resolution::Time1_16 => 2,
            Resolution::Time1_32 => 1,
        }
    }

    pub fn next_down(&self) -> Resolution {
        match self {
            Resolution::Time1_32 => Resolution::Time1_16,
            Resolution::Time1_16 => Resolution::Time1_8,
            Resolution::Time1_8 => Resolution::Time1_4,
            Resolution::Time1_4 => Resolution::Time1_4,
        }
    }

    pub fn next_up(&self) -> Resolution {
        match self {
            Resolution::Time1_4 => Resolution::Time1_8,
            Resolution::Time1_8 => Resolution::Time1_16,
            Resolution::Time1_16 => Resolution::Time1_32,
            Resolution::Time1_32 => Resolution::Time1_32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_as_str() {
        assert_eq!(Resolution::Time1_4.as_str(), "1/4");
        assert_eq!(Resolution::Time1_8.as_str(), "1/8");
        assert_eq!(Resolution::Time1_16.as_str(), "1/16");
        assert_eq!(Resolution::Time1_32.as_str(), "1/32");
    }

    #[test]
    fn test_bar_length_in_beats() {
        assert_eq!(Resolution::Time1_4.bar_length_in_beats(), 4);
        assert_eq!(Resolution::Time1_8.bar_length_in_beats(), 8);
        assert_eq!(Resolution::Time1_16.bar_length_in_beats(), 16);
        assert_eq!(Resolution::Time1_32.bar_length_in_beats(), 32);
    }

    #[test]
    fn test_duration_b32() {
        assert_eq!(Resolution::Time1_4.duration_b32(), 8);  // 1/4 note = 8 32nd notes
        assert_eq!(Resolution::Time1_8.duration_b32(), 4);  // 1/8 note = 4 32nd notes
        assert_eq!(Resolution::Time1_16.duration_b32(), 2); // 1/16 note = 2 32nd notes
        assert_eq!(Resolution::Time1_32.duration_b32(), 1); // 1/32 note = 1 32nd note
    }

    #[test]
    fn test_next_down() {
        assert_eq!(Resolution::Time1_32.next_down(), Resolution::Time1_16);
        assert_eq!(Resolution::Time1_16.next_down(), Resolution::Time1_8);
        assert_eq!(Resolution::Time1_8.next_down(), Resolution::Time1_4);
        // At limit - doesn't change
        assert_eq!(Resolution::Time1_4.next_down(), Resolution::Time1_4);
    }

    #[test]
    fn test_next_up() {
        assert_eq!(Resolution::Time1_4.next_up(), Resolution::Time1_8);
        assert_eq!(Resolution::Time1_8.next_up(), Resolution::Time1_16);
        assert_eq!(Resolution::Time1_16.next_up(), Resolution::Time1_32);
        // At limit - doesn't change
        assert_eq!(Resolution::Time1_32.next_up(), Resolution::Time1_32);
    }
}
