use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

pub enum TuiEvent {
    Key(KeyEvent),
    Tick,
}

pub fn next_event(timeout: Duration) -> Result<TuiEvent> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => Ok(TuiEvent::Key(key)),
            _ => Ok(TuiEvent::Tick),
        }
    } else {
        Ok(TuiEvent::Tick)
    }
}

pub fn is_quit_key(code: KeyCode, modifiers: KeyModifiers) -> bool {
    matches!(code, KeyCode::Char('q'))
        || (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL))
}
