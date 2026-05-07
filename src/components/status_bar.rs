use ratatui::{
    Frame,
    prelude::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Paragraph},
};
use ratatui_themes::Style;

use crate::{app_context, components::shortcut::Shortcut, screen::Screen};

use super::Renderable;

pub struct StatusBar {
    pub screen: Screen,
}

impl StatusBar {
    fn shortcuts_line<'b>(shortcuts: &[Shortcut<'b>]) -> Line<'b> {
        let mut spans = Vec::new();

        for (i, shortcut) in shortcuts.iter().enumerate() {
            spans.extend(shortcut.spans());

            if i < shortcuts.len() - 1 {
                spans.push(Span::raw(" "));
            }
        }

        Line::from(spans)
    }

    fn status(&self) -> &str {
        match self.screen {
            Screen::Error => "Error",
            Screen::PoolOverview => "Normal",
            Screen::PoolPicker => "Pick",
            Screen::Help => "Help",
        }
    }
}

impl Renderable for StatusBar {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let palette = app_context::theme().palette();

        frame.render_widget(
            Block::default().style(Style::default().bg(palette.selection)),
            area,
        );

        let mut shortcuts = Vec::new();

        if self.screen == Screen::Help {
            shortcuts.push(Shortcut {
                label: "close",
                shortcut: "Esc",
            });
            shortcuts.push(Shortcut {
                label: "close",
                shortcut: "?",
            });
        } else {
            if self.screen != Screen::Error {
                shortcuts.push(Shortcut {
                    label: "navigate",
                    shortcut: "⇅",
                });
            }

            if self.screen == Screen::PoolPicker {
                shortcuts.push(Shortcut {
                    label: "select",
                    shortcut: "↵",
                });
            }

            if self.screen == Screen::PoolOverview {
                shortcuts.push(Shortcut {
                    label: "units",
                    shortcut: "u",
                });
                shortcuts.push(Shortcut {
                    label: "theme",
                    shortcut: "t",
                });
            }

            shortcuts.push(Shortcut {
                label: "refresh",
                shortcut: "r",
            });
            shortcuts.push(Shortcut {
                label: "help",
                shortcut: "?",
            });
            shortcuts.push(Shortcut {
                label: "quit",
                shortcut: "q",
            });
        }

        let mut shortcuts_line = StatusBar::shortcuts_line(&shortcuts);

        shortcuts_line.spans.insert(
            0,
            Span::raw(format!(" {} ", self.status().to_uppercase()))
                .bold()
                .fg(palette.info)
                .reversed(),
        );
        shortcuts_line.spans.insert(1, Span::raw(" "));

        frame.render_widget(Paragraph::new(shortcuts_line).block(Block::default()), area);
    }
}
