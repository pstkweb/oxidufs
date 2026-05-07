use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Paragraph, Tabs},
};
use ratatui_themes::{Color, Style};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, FromRepr};

use crate::{
    app_context::{self},
    app_state::AppState,
    components::{Dispatcher, Error, ErrorType, PoolOverview},
    keymap::Action,
};

#[derive(Default, Debug, Clone, Copy, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "Pool Overview")]
    PoolOverview,
    #[strum(to_string = "File Distribution")]
    FileDistribution,
    #[strum(to_string = "Health")]
    Health,
    #[strum(to_string = "Rebalance")]
    Rebalance,
    #[strum(to_string = "Policy")]
    Policy,
}

impl SelectedTab {
    fn title(self) -> Line<'static> {
        let shortcut_index = self as usize + 1;

        Line::from(vec![
            Span::styled(
                format!(" F{shortcut_index} "),
                Style::default()
                    .bold()
                    .bg(app_context::theme().palette().info),
            ),
            Span::raw(" "),
            Span::raw(format!("{self}")),
        ])
    }
}

#[derive(Debug)]
pub struct MainTab {
    pool_overview: PoolOverview,
    selected_tab: SelectedTab,
}

impl Default for MainTab {
    fn default() -> Self {
        Self {
            pool_overview: PoolOverview::default(),
            selected_tab: SelectedTab::PoolOverview,
        }
    }
}

impl MainTab {
    pub fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        state: &AppState,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Fill(1)])
            .split(area);
        let [header_area, content_area] = chunks[..] else {
            unreachable!()
        };

        frame.render_widget(
            Tabs::new(SelectedTab::iter().map(SelectedTab::title))
                .select(self.selected_tab as usize)
                .highlight_style(Style::default().bold().fg(Color::White)),
            header_area,
        );

        let wip_widget = Paragraph::new("Work in progress.").alignment(Alignment::Center);
        let wip_area = content_area.centered(Constraint::Percentage(50), Constraint::Length(1));

        match self.selected_tab {
            SelectedTab::PoolOverview => self.pool_overview.render(frame, content_area, state),
            _ => frame.render_widget(wip_widget, wip_area),
        }
    }
}

impl Dispatcher for MainTab {
    fn dispatch(&mut self, action: &Action, state: &mut AppState) -> Option<Error> {
        let mut error = None;

        if let SelectedTab::PoolOverview = self.selected_tab {
            error = self.pool_overview.dispatch(action, state);
        }

        if let Action::SelectMainTab(index) = action {
            if let Some(tab) = SelectedTab::from_repr(*index) {
                self.selected_tab = tab;
            } else {
                error = Some(Error {
                    error_type: ErrorType::Error,
                    message: "Unable to find something to display here.".to_string(),
                    tips: vec![],
                });
            }
        }

        error
    }
}
