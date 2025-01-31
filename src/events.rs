use crate::mode::Mode;
use crossterm::event::{poll, read, Event, KeyCode};
use std::io;
use std::sync::{Arc, Mutex};
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
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
}

pub fn capture_input(
    tx: &mpsc::Sender<InputEvent>,
    mode_lock: &Arc<Mutex<Mode>>,
) -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    loop {
        if poll(Duration::from_millis(500))? {
            if let Event::Key(event) = read()? {
                let mode = *mode_lock.lock().unwrap();
                match event.code {
                    KeyCode::Char('3') => tx.send(InputEvent::ToggleMode).unwrap(),
                    KeyCode::Char('4') => tx.send(InputEvent::PlayerTogglePlayback).unwrap(),
                    KeyCode::Char('p') => {
                        tx.send(InputEvent::Quit).unwrap();
                        break;
                    }
                    KeyCode::Up => {
                        let event = match mode {
                            Mode::Normal => InputEvent::ViewerOctaveIncrease,
                            Mode::Select | Mode::Insert => InputEvent::CursorUp,
                        };
                        tx.send(event).unwrap();
                    }
                    KeyCode::Down => {
                        let event = match mode {
                            Mode::Normal => InputEvent::ViewerOctaveDecrease,
                            Mode::Select | Mode::Insert => InputEvent::CursorDown,
                        };
                        tx.send(event).unwrap();
                    }
                    KeyCode::Left => {
                        let event = match mode {
                            Mode::Normal => InputEvent::ViewerBarPrevious,
                            Mode::Select | Mode::Insert => InputEvent::CursorLeft,
                        };
                        tx.send(event).unwrap();
                    }
                    KeyCode::Right => {
                        let event = match mode {
                            Mode::Normal => InputEvent::ViewerBarNext,
                            Mode::Select | Mode::Insert => InputEvent::CursorRight,
                        };
                        tx.send(event).unwrap();
                    }
                    KeyCode::Char('[') => tx.send(InputEvent::ViewerResolutionDecrease).unwrap(),
                    KeyCode::Char(']') => tx.send(InputEvent::ViewerResolutionIncrease).unwrap(),
                    _ => (),
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
