use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    prelude::Rect,
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};
use ratatui_themes::Style;

use crate::app_context;

pub enum ErrorType {
    Error,
    Warning,
}

impl std::fmt::Debug for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::Error => write!(f, "Error"),
            ErrorType::Warning => write!(f, "Warning"),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub error_type: ErrorType,
    pub message: String,
    pub tips: Vec<Line<'static>>,
}

impl Error {
    /// Minimum height needed to render this error widget (including border).
    pub fn height(&self) -> u16 {
        4 + self.tips.len() as u16
    }
}

impl super::Renderable for Error {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let theme_palette = app_context::theme().palette();

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(format!(
                " {} ",
                match self.error_type {
                    ErrorType::Error => "error",
                    ErrorType::Warning => "warning",
                }
            ))
            .title_style(Style::default().fg(match self.error_type {
                ErrorType::Error => theme_palette.error,
                ErrorType::Warning => theme_palette.warning,
            }));
        let block_layout = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(self.tips.len() as u16),
        ])
        .split(area);

        let message = Paragraph::new(self.message.clone());
        let tips = Paragraph::new(
            self.tips
                .iter()
                .map(|t| {
                    let mut prefix = vec![Span::styled(
                        "tip  ",
                        Style::default().fg(theme_palette.info),
                    )];
                    prefix.extend(t.spans.iter().cloned());

                    Line::from(prefix)
                })
                .collect::<Vec<Line>>(),
        );

        frame.render_widget(message, block_layout[0]);
        frame.render_widget(tips, block_layout[2]);
        frame.render_widget(block, area);
    }
}
