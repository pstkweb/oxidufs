use ratatui::text::Span;
use ratatui_themes::{Color, Style};

use crate::app_context;

#[derive(Debug)]
pub struct Shortcut<'a> {
    pub label: &'a str,
    pub shortcut: &'a str,
}

impl<'a> Shortcut<'a> {
    pub fn spans(&self) -> Vec<Span<'a>> {
        let palette = app_context::theme().palette();

        vec![
            Span::styled(self.shortcut, Style::default().fg(Color::White).bold()),
            Span::raw(" "),
            Span::styled(self.label, Style::default().fg(palette.muted)),
        ]
    }
}
