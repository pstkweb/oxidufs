use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Cell, Clear, Row, Table},
};
use ratatui_themes::{Color, Style};

use crate::app_context;

#[derive(Debug, Default)]
pub struct Help;

struct Section {
    title: &'static str,
    bindings: &'static [(&'static str, &'static str)],
}

const SECTIONS: &[Section] = &[
    Section {
        title: "Navigation",
        bindings: &[
            ("j / ↓", "Move selection down"),
            ("k / ↑", "Move selection up"),
            ("Enter", "Select pool (in pool picker)"),
            ("F1–F5", "Switch tab"),
        ],
    },
    Section {
        title: "Display",
        bindings: &[("u", "Toggle unit (SI / binary)"), ("t", "Cycle theme")],
    },
    Section {
        title: "App",
        bindings: &[
            ("r", "Refresh"),
            ("?", "Toggle this help"),
            ("Esc", "Close help"),
            ("q", "Quit"),
        ],
    },
];

impl Help {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let palette = app_context::theme().palette();
        let label_style = Style::default().fg(palette.muted);

        let popup = area.centered(Constraint::Length(48), Constraint::Length(self.height()));

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" Help ")
            .title_style(Style::default().fg(palette.info));

        let inner = block.inner(popup);

        frame.render_widget(Clear, popup);
        frame.render_widget(block, popup);

        let mut rows: Vec<Row<'static>> = Vec::new();

        for (i, section) in SECTIONS.iter().enumerate() {
            if i > 0 {
                rows.push(Row::new(vec![Cell::from(""), Cell::from("")]));
            }

            rows.push(Row::new(vec![
                Cell::from(""),
                Cell::from(Span::styled(
                    section.title,
                    Style::default().fg(palette.accent).bold(),
                )),
            ]));

            for (key, label) in section.bindings {
                rows.push(Row::new(vec![
                    Cell::from(Line::from(Span::styled(
                        *key,
                        Style::default().fg(Color::White),
                    )))
                    .style(Style::default().fg(Color::White)),
                    Cell::from(Span::styled(*label, label_style)),
                ]));
            }
        }

        let table =
            Table::new(rows, [Constraint::Length(10), Constraint::Min(0)]).column_spacing(2);

        frame.render_widget(table, inner);
    }

    fn height(&self) -> u16 {
        let body: usize = SECTIONS.iter().map(|s| 1 + s.bindings.len()).sum::<usize>()
            + SECTIONS.len().saturating_sub(1);

        body as u16 + 2
    }
}
