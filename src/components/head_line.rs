use ratatui::{
    style::Style,
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use crate::app_context;

#[derive(Debug, Default)]
pub struct HeadLine;

impl super::Renderable for HeadLine {
    fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        let palette = app_context::theme().palette();

        let text = Text::from(vec![Line::from(vec![
            Span::styled(" OxiDuFs", Style::default().bold().fg(palette.accent)),
            Span::styled(
                " │ a TUI for your mergerfs universe │ v0.1.0",
                Style::default().fg(palette.muted),
            ),
        ])]);

        frame.render_widget(
            Paragraph::new(text).style(Style::default().bg(palette.selection)),
            area,
        );
    }
}
