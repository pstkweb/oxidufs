use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::screen::Screen;

#[derive(Debug)]
pub enum Action {
    CycleTheme,
    Help,
    NextItem,
    PrevItem,
    Quit,
    Refresh,
    Select,
    SelectMainTab(usize),
    ToggleUnit,
}

pub fn key_to_action(key: KeyEvent, screen: &Screen) -> Option<Action> {
    match (key.code, key.modifiers, screen) {
        (KeyCode::Char('?'), _, _) => Some(Action::Help),
        (KeyCode::Esc, _, Screen::Help) => Some(Action::Help),
        (_, _, Screen::Help) => None,
        (KeyCode::Char('q'), KeyModifiers::NONE, _) => Some(Action::Quit),
        (KeyCode::Char('t'), KeyModifiers::NONE, _) => Some(Action::CycleTheme),
        (KeyCode::Char('r'), KeyModifiers::NONE, _) => Some(Action::Refresh),
        (KeyCode::Char('u'), KeyModifiers::NONE, Screen::PoolOverview) => Some(Action::ToggleUnit),
        (KeyCode::Char('j') | KeyCode::Down, KeyModifiers::NONE, _) => Some(Action::NextItem),
        (KeyCode::Char('k') | KeyCode::Up, KeyModifiers::NONE, _) => Some(Action::PrevItem),
        (KeyCode::Enter, KeyModifiers::NONE, Screen::PoolPicker) => Some(Action::Select),
        (KeyCode::F(1), KeyModifiers::NONE, Screen::PoolOverview) => Some(Action::SelectMainTab(0)),
        (KeyCode::F(2), KeyModifiers::NONE, Screen::PoolOverview) => Some(Action::SelectMainTab(1)),
        (KeyCode::F(3), KeyModifiers::NONE, Screen::PoolOverview) => Some(Action::SelectMainTab(2)),
        (KeyCode::F(4), KeyModifiers::NONE, Screen::PoolOverview) => Some(Action::SelectMainTab(3)),
        (KeyCode::F(5), KeyModifiers::NONE, Screen::PoolOverview) => Some(Action::SelectMainTab(4)),
        _ => None,
    }
}
