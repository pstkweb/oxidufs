use humansize::FormatSize;
use ratatui::{
    layout::{Constraint, Layout},
    symbols,
    widgets::{Block, BorderType, Cell, HighlightSpacing, Row, Table, TableState},
};
use ratatui_themes::{Color, Style};

use crate::{
    app_context,
    app_state::AppState,
    components::{Dispatcher, Error},
    data::pool::PoolData,
    keymap::Action,
    theme::usage_bar,
};

#[derive(Debug)]
pub struct PoolPicker {
    pools_selector_state: TableState,
}

impl PoolPicker {
    pub fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        state: &AppState,
    ) {
        let theme_palette = app_context::theme().palette();

        let table = Table::new(
            Self::pool_rows(
                state
                    .pools
                    .iter()
                    .filter_map(|p| PoolData::load(&p.mount_point, state))
                    .collect::<Vec<PoolData>>()
                    .as_slice(),
            ),
            [
                Constraint::Min(0),
                Constraint::Min(0),
                Constraint::Min(0),
                Constraint::Min(24),
            ],
        )
        .column_spacing(2)
        .highlight_symbol(format!(" {} ", symbols::scrollbar::DOUBLE_HORIZONTAL.end))
        .highlight_spacing(HighlightSpacing::Always)
        .row_highlight_style(Style::default().bg(theme_palette.selection));

        let chunks = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Length(state.pools.len() as u16 + 2),
            Constraint::Fill(1),
        ])
        .split(area);
        let [_, pools_list, _] = chunks[..] else {
            unreachable!()
        };

        frame.render_stateful_widget(
            table.block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(" Multiple pools detected ")
                    .title_style(Style::default().fg(theme_palette.warning)),
            ),
            pools_list,
            &mut self.pools_selector_state,
        );
    }

    fn pool_rows(pools: &[PoolData]) -> Vec<Row<'static>> {
        let theme_palette = app_context::theme().palette();
        let unit_format = app_context::size_format().format_options();

        pools
            .iter()
            .map(|pool| {
                Row::new(vec![
                    Cell::from(pool.mount_point.to_string())
                        .style(Style::default().fg(Color::White)),
                    Cell::from(pool.summary().to_string())
                        .style(Style::default().fg(theme_palette.secondary)),
                    Cell::from(
                        pool.disks
                            .iter()
                            .map(|d| d.total_bytes)
                            .sum::<u64>()
                            .format_size(unit_format),
                    )
                    .style(Style::default().fg(Color::Green)),
                    Cell::from(usage_bar(pool.use_pct(), 20)),
                ])
            })
            .collect()
    }
}

impl Default for PoolPicker {
    fn default() -> Self {
        let mut table_state = TableState::default();
        table_state.select_first();

        Self {
            pools_selector_state: table_state,
        }
    }
}

impl Dispatcher for PoolPicker {
    fn dispatch(&mut self, action: &Action, state: &mut AppState) -> Option<Error> {
        match action {
            Action::NextItem => self.pools_selector_state.select_next(),
            Action::PrevItem => self.pools_selector_state.select_previous(),
            Action::Select => state.set_current_pool(
                state.pools[self.pools_selector_state.selected().unwrap_or_default()]
                    .mount_point
                    .to_string(),
            ),
            _ => (),
        }
        None
    }
}
