use crossterm::event::{poll, read, Event, KeyCode};
use std::io;
use std::sync::mpsc;
use std::time::Duration;

pub enum InputEvent {
    ViewerBarNext,
    ViewerBarPrevious,
    ViewerResolutionIncrease,
    ViewerResolutionDecrease,
    ViewerOctaveIncrease,
    ViewerOctaveDecrease,
    PlayerTogglePlayback,
    Quit,
    PlayerBeatChange(u64),
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    InsertNote,
    StartLongNote,
    EndLongNote,
    Cancel,
    Yank,
    Cut,
    Paste,
    Delete,
    ToggleLoopMode,
    SetLoopTimes,
    SaveSong,
    SelectIn,
}

pub fn capture_input(tx: &mpsc::Sender<InputEvent>) -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut shift_pressed = false;

    loop {
        if poll(Duration::from_millis(500))? {
            if let Event::Key(event) = read()? {
                match event.code {
                    // Shift key (right side button)
                    KeyCode::Char('`') => shift_pressed = !shift_pressed,
                    
                    // Arrow keys - Cursor movement or Viewport navigation
                    KeyCode::Left => {
                        tx.send(if shift_pressed {
                            InputEvent::ViewerBarPrevious
                        } else {
                            InputEvent::CursorLeft
                        }).unwrap();
                    }
                    KeyCode::Right => {
                        tx.send(if shift_pressed {
                            InputEvent::ViewerBarNext
                        } else {
                            InputEvent::CursorRight
                        }).unwrap();
                    }
                    KeyCode::Up => {
                        tx.send(if shift_pressed {
                            InputEvent::ViewerResolutionIncrease
                        } else {
                            InputEvent::CursorUp
                        }).unwrap();
                    }
                    KeyCode::Down => {
                        tx.send(if shift_pressed {
                            InputEvent::ViewerResolutionDecrease
                        } else {
                            InputEvent::CursorDown
                        }).unwrap();
                    }
                    
                    // Essential controls
                    KeyCode::Esc => tx.send(InputEvent::Cancel).unwrap(),
                    
                    // Left side keys - Main editing
                    KeyCode::Char('1') => tx.send(InputEvent::InsertNote).unwrap(),
                    KeyCode::Char('2') => tx.send(InputEvent::StartLongNote).unwrap(),
                    KeyCode::Char('3') => tx.send(InputEvent::EndLongNote).unwrap(),
                    KeyCode::Char('4') => tx.send(InputEvent::Delete).unwrap(),
                    
                    // Second row - Loop
                    KeyCode::Char('e') => tx.send(InputEvent::ToggleLoopMode).unwrap(),
                    KeyCode::Char('r') => tx.send(InputEvent::SetLoopTimes).unwrap(),
                    
                    // Third row - Clipboard
                    KeyCode::Char('a') => tx.send(InputEvent::Yank).unwrap(),
                    KeyCode::Char('s') => tx.send(InputEvent::Cut).unwrap(),
                    KeyCode::Char('d') => tx.send(InputEvent::Paste).unwrap(),
                    KeyCode::Char('f') => tx.send(InputEvent::SaveSong).unwrap(),
                    
                    // Playback control
                    KeyCode::Char('\\') => tx.send(InputEvent::PlayerTogglePlayback).unwrap(),
                    
                    // Add these new mappings - using Q and W keys for selection
                    KeyCode::Char('q') => tx.send(InputEvent::SelectIn).unwrap(),
                    
                    KeyCode::Char('p') => {
                        tx.send(InputEvent::Quit).unwrap();
                        break;
                    }
                    _ => (),
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
