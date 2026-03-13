use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::screen::Screen;

#[derive(Debug)]
pub enum Action {
    ToggleUnit,
    CycleTheme,
    SelectMainTab(usize),
    NextDisk,
    PrevDisk,
    Help,
    Refresh,
    Quit,
}

pub fn key_to_action(key: KeyEvent, _screen: &Screen) -> Option<Action> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::NONE) => Some(Action::Quit),
        (KeyCode::Char('t'), KeyModifiers::NONE) => Some(Action::CycleTheme),
        (KeyCode::Char('u'), KeyModifiers::NONE) => Some(Action::ToggleUnit),
        (KeyCode::Char('j') | KeyCode::Down, KeyModifiers::NONE) => Some(Action::NextDisk),
        (KeyCode::Char('k') | KeyCode::Up, KeyModifiers::NONE) => Some(Action::PrevDisk),
        (KeyCode::F(1), KeyModifiers::NONE) => Some(Action::SelectMainTab(0)),
        (KeyCode::F(2), KeyModifiers::NONE) => Some(Action::SelectMainTab(1)),
        (KeyCode::F(3), KeyModifiers::NONE) => Some(Action::SelectMainTab(2)),
        (KeyCode::F(4), KeyModifiers::NONE) => Some(Action::SelectMainTab(3)),
        (KeyCode::F(5), KeyModifiers::NONE) => Some(Action::SelectMainTab(4)),
        _ => None,
    }
}
