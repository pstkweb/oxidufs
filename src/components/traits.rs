use ratatui::{Frame, layout::Rect, widgets::Block};

use crate::keymap::Action;

pub trait Renderable {
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

impl<'a> Renderable for Block<'a> {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(&*self, area);
    }
}

pub trait Dispatcher {
    fn dispatch(&mut self, action: &Action);
}
