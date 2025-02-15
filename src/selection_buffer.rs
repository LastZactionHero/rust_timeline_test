use crate::score::Score;

#[derive(Debug)]
pub enum SelectionBuffer {
    None,
    Score(Score),
}
