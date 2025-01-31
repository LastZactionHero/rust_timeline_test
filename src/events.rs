use crossterm::event::{poll, read, Event, KeyCode};
use std::io;
use std::{sync::mpsc, time::Duration};

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
    ToggleMode,
}

pub fn capture_input(tx: &mpsc::Sender<InputEvent>) -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    loop {
        if poll(Duration::from_millis(500))? {
            if let Event::Key(event) = read()? {
                match event.code {
                    KeyCode::Char('3') => tx.send(InputEvent::ToggleMode).unwrap(),
                    KeyCode::Char('4') => tx.send(InputEvent::PlayerTogglePlayback).unwrap(),
                    // Legacy
                    KeyCode::Char('p') => {
                        tx.send(InputEvent::Quit).unwrap();
                        break;
                    }
                    KeyCode::Up => tx.send(InputEvent::ViewerOctaveIncrease).unwrap(),
                    KeyCode::Down => tx.send(InputEvent::ViewerOctaveDecrease).unwrap(),
                    KeyCode::Left => tx.send(InputEvent::ViewerBarPrevious).unwrap(),
                    KeyCode::Right => tx.send(InputEvent::ViewerBarNext).unwrap(),
                    KeyCode::Char('[') => tx.send(InputEvent::ViewerResolutionDecrease).unwrap(),
                    KeyCode::Char(']') => tx.send(InputEvent::ViewerResolutionIncrease).unwrap(),
                    KeyCode::Char(' ') => tx.send(InputEvent::PlayerTogglePlayback).unwrap(),
                    _ => (),
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
