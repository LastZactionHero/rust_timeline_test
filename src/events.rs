pub enum InputEvent {
    ViewerOctaveIncrease,
    ViewerOctaveDecrease,
    ViewerBarNext,
    ViewerBarPrevious,
    ViewerResolutionIncrease,
    ViewerResolutionDecrease,
    PlayerTogglePlayback,
    Quit,
    PlayerBeatChange(u64),
    PlayheadOutOfViewport,
}
