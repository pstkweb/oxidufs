use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Paragraph},
};
use ratatui_themes::Style;

use crate::{app_context, components::shortcut::Shortcut};

#[derive(Default)]
pub struct StatusBar;

impl StatusBar {
    fn shortcuts_line<'a>(shortcuts: &[Shortcut<'a>]) -> Line<'a> {
        let mut spans = Vec::new();

        for (i, shortcut) in shortcuts.iter().enumerate() {
            spans.extend(shortcut.spans());

            if i < shortcuts.len() - 1 {
                spans.push(Span::raw(" "));
            }
        }

        Line::from(spans)
    }
}

impl super::Renderable for StatusBar {
    fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        let palette = app_context::theme().palette();

        frame.render_widget(
            Block::default().style(Style::default().bg(palette.selection)),
            area,
        );

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Min(0), Constraint::Min(10)])
            .split(area);

        let mut shortcuts = StatusBar::shortcuts_line(&[
            Shortcut {
                label: "navigate",
                shortcut: '⇅',
            },
            Shortcut {
                label: "expand",
                shortcut: '↵',
            },
            Shortcut {
                label: "refresh",
                shortcut: 'r',
            },
            Shortcut {
                label: "units",
                shortcut: 'u',
            },
            Shortcut {
                label: "theme",
                shortcut: 't',
            },
            Shortcut {
                label: "quit",
                shortcut: 'q',
            },
        ]);

        shortcuts
            .spans
            .insert(0, Span::raw(" NORMAL ").bold().fg(palette.info).reversed());
        shortcuts.spans.insert(1, Span::raw(" "));

        frame.render_widget(Paragraph::new(shortcuts).block(Block::default()), chunks[0]);
        frame.render_widget(
            Paragraph::new("last refresh: -- ")
                .style(Style::default().fg(palette.muted))
                .right_aligned(),
            chunks[2],
        );
    }
}
