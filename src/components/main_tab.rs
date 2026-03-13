use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::Tabs,
};
use ratatui_themes::{Color, Style};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, FromRepr};

use crate::{
    app_context::{self},
    components::{Dispatcher, PoolOverview, Renderable},
    keymap::Action,
};

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
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

pub struct MainTab {
    selected_tab: SelectedTab,
    pool_overview: PoolOverview,
}

impl Default for MainTab {
    fn default() -> Self {
        Self {
            selected_tab: SelectedTab::PoolOverview,
            pool_overview: PoolOverview::default(),
        }
    }
}

impl Renderable for MainTab {
    fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Fill(1)])
            .split(area);
        let [header_area, content_area] = &*chunks else {
            unreachable!()
        };

        frame.render_widget(
            Tabs::new(SelectedTab::iter().map(SelectedTab::title))
                .select(self.selected_tab as usize)
                .highlight_style(Style::default().bold().fg(Color::White)),
            *header_area,
        );

        match self.selected_tab {
            SelectedTab::PoolOverview => self.pool_overview.render(frame, *content_area),
            _ => {}
        }
    }
}

impl Dispatcher for MainTab {
    fn dispatch(&mut self, action: &Action) {
        match self.selected_tab {
            SelectedTab::PoolOverview => self.pool_overview.dispatch(action),
            _ => {}
        }

        match action {
            Action::SelectMainTab(index) => {
                if let Some(tab) = SelectedTab::from_repr(*index) {
                    self.selected_tab = tab;
                } else {
                    // TODO show error
                }
            }
            _ => {}
        }
    }
}
