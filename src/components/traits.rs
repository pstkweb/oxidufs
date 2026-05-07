use ratatui::{Frame, layout::Rect};

use crate::{app_state::AppState, components::Error, keymap::Action};

pub trait Renderable {
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

pub trait Dispatcher {
    fn dispatch(&mut self, action: &Action, state: &mut AppState) -> Option<Error>;
}
