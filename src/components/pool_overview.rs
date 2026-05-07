use humansize::{FormatSize, FormatSizeOptions};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols,
    text::{Line, Span},
    widgets::{Block, BorderType, Cell, HighlightSpacing, Row, Table, TableState},
};
use ratatui_themes::{Color, Style, ThemePalette};

use crate::{app_context, components::Dispatcher, keymap::Action};
use crate::{
    app_state::AppState,
    components::{
        Renderable,
        error::{Error, ErrorType},
    },
    data::pool::{DiskData, PoolData},
    model::pool::PoolStats,
    theme::{usage_bar, usage_color},
};

#[derive(Debug)]
pub struct PoolOverview {
    disks_selector_state: TableState,
}

#[derive(Debug)]
struct UiContext {
    palette: ThemePalette,
    unit_format: FormatSizeOptions,
    label_style: Style,
}

impl UiContext {
    fn new() -> Self {
        let palette = app_context::theme().palette();

        Self {
            palette,
            unit_format: app_context::size_format().format_options(),
            label_style: Style::default().fg(palette.muted),
        }
    }
}

impl PoolOverview {
    pub fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        state: &AppState,
    ) {
        let context = UiContext::new();
        let chunks = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Min(3),
            Constraint::Fill(1),
        ])
        .split(area);
        let [pool_summary, disks_table, disk_details] = chunks[..] else {
            unreachable!()
        };

        let Some(mount_point) = state.current_pool.as_ref() else {
            let em_style = Style::default().fg(app_context::theme().palette().secondary);
            let mut err_widget = Error {
                error_type: ErrorType::Error,
                message: "no mergerfs mount points detected on this system".to_string(),
                tips: vec![
                    Line::from(vec![
                        Span::raw("specify a mount point explicitly: "),
                        Span::styled("oxidufs /mnt/pool", em_style),
                    ]),
                    Line::from(vec![
                        Span::raw("is mergerfs installed? try "),
                        Span::styled("which mergerfs", em_style),
                    ]),
                ],
            };

            err_widget.render(frame, area);

            return;
        };

        let Some(data) = PoolData::load(mount_point.as_str(), state) else {
            let message = match state.fs_type_at(mount_point.as_str()) {
                Some(fs) => format!("{mount_point} is not a mergerfs mount point (it is {fs})"),
                None => format!("{mount_point} is not a mergerfs mount point"),
            };
            let mut err_widget = Error {
                error_type: ErrorType::Error,
                message,
                tips: vec![Line::from(
                    "point oxidufs at your pool root, not an underlying member disk",
                )],
            };

            err_widget.render(frame, area);

            return;
        };

        self.render_pool_summary(frame, pool_summary, &context, &data);
        self.render_disks_table(frame, disks_table, &context, &data);
        self.render_disk_details(frame, disk_details, &context, &data);
    }

    fn render_pool_summary(
        &self,
        frame: &mut Frame,
        area: Rect,
        context: &UiContext,
        data: &PoolData,
    ) {
        let stats = PoolStats::from_pool(data).unwrap_or_default();
        let theme_palette = context.palette;
        let unit_format = context.unit_format;
        let label_style = context.label_style;

        let pool_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" Pool ")
            .title_style(Style::default().fg(theme_palette.info));

        let pool_block_layout = Layout::vertical([Constraint::Length(2), Constraint::Length(1)])
            .split(pool_block.inner(area.centered_vertically(Constraint::Length(5))));
        let [table, fuse_line] = pool_block_layout[..] else {
            unreachable!()
        };

        let pool_table = Table::new(
            [
                Row::new(vec![
                    Cell::from(Line::from(vec![
                        Span::styled(format!("{:<8}", "mount"), label_style),
                        Span::styled(
                            data.mount_point.clone(),
                            Style::default().fg(theme_palette.info),
                        ),
                    ])),
                    Cell::from(Line::from(vec![
                        Span::styled(format!("| {:<10}", "policy"), label_style),
                        Span::styled(
                            data.policy.clone(),
                            Style::default().fg(theme_palette.warning),
                        ),
                    ])),
                    Cell::from(Line::from(vec![
                        Span::styled(format!("| {:<10}", "disks"), label_style),
                        Span::styled(
                            data.disks.len().to_string(),
                            Style::default().fg(Color::White),
                        ),
                    ])),
                ]),
                Row::new(vec![
                    Cell::from(Line::from(vec![
                        Span::styled(format!("{:<8}", "total"), label_style),
                        Span::styled(
                            stats.total_bytes.format_size(unit_format),
                            Style::default().fg(Color::White),
                        ),
                    ])),
                    Cell::from(Line::from(vec![
                        Span::styled(format!("| {:<10}", "used"), label_style),
                        Span::styled(
                            stats.used_bytes.format_size(unit_format),
                            Style::default().fg(theme_palette.warning),
                        ),
                    ])),
                    Cell::from(Line::from(vec![
                        Span::styled(format!("| {:<10}", "free"), label_style),
                        Span::styled(
                            stats.free_bytes.format_size(unit_format),
                            Style::default().fg(theme_palette.success),
                        ),
                    ])),
                ]),
            ],
            [Constraint::Min(0), Constraint::Min(0), Constraint::Min(0)],
        );
        let pool_fuse_line = Line::from(vec![
            Span::styled(format!("{:<8}", "options"), label_style),
            Span::styled(
                data.fuse_options.to_string(),
                Style::default().fg(Color::White),
            ),
        ]);

        frame.render_widget(pool_block, area.centered_vertically(Constraint::Length(5)));
        frame.render_widget(pool_table, table);
        frame.render_widget(pool_fuse_line, fuse_line);
    }

    fn render_disks_table(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        context: &UiContext,
        data: &PoolData,
    ) {
        let stats = PoolStats::from_pool(data).unwrap_or_default();
        let rows = Self::disk_rows(&data.disks, context);
        let theme_palette = context.palette;

        if rows.is_empty() {
            frame.render_widget(
                Span::from("Pool is empty").bold().fg(theme_palette.muted),
                area,
            );

            return;
        }

        let unit_format = context.unit_format;
        let table = Table::new(
            rows,
            [
                Constraint::Min(0),
                Constraint::Min(0),
                Constraint::Min(0),
                Constraint::Min(0),
                Constraint::Min(0),
                Constraint::Min(0),
                Constraint::Min(24),
            ],
        )
        .header(
            Row::new(vec![
                "DEVICE", "MOUNT", "FS", "SIZE", "USED", "FREE", "USAGE",
            ])
            .style(Style::default().bg(theme_palette.selection)),
        )
        .footer(
            Row::new(vec![
                Cell::from("TOTAL"),
                Cell::from(""),
                Cell::from(""),
                Cell::from(stats.total_bytes.format_size(unit_format))
                    .style(Style::default().fg(Color::White)),
                Cell::from(stats.used_bytes.format_size(unit_format))
                    .style(Style::default().fg(usage_color(stats.used_pct))),
                Cell::from(stats.free_bytes.format_size(unit_format))
                    .style(Style::default().fg(Color::Green)),
                Cell::from(usage_bar(stats.used_pct, 20)),
            ])
            .style(Style::default().bg(theme_palette.selection)),
        )
        .column_spacing(2)
        .highlight_symbol(format!(" {} ", symbols::scrollbar::DOUBLE_HORIZONTAL.end))
        .highlight_spacing(HighlightSpacing::Always)
        .row_highlight_style(Style::default().bg(theme_palette.selection));

        frame.render_stateful_widget(table, area, &mut self.disks_selector_state);
    }

    fn render_disk_details(
        &self,
        frame: &mut Frame,
        area: Rect,
        context: &UiContext,
        data: &PoolData,
    ) {
        let theme_palette = context.palette;
        let label_style = context.label_style;

        let selected = self.disks_selector_state.selected().unwrap_or_default();

        let Some(disk) = data.disks.get(selected) else {
            let mut err_widget = Error {
                error_type: ErrorType::Warning,
                message: "no disk data available".to_string(),
                tips: vec![Line::from(
                    "press r to refresh — the pool may have no member disks yet",
                )],
            };

            err_widget.render(
                frame,
                area.centered_vertically(Constraint::Length(err_widget.height())),
            );

            return;
        };

        let disk_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(format!(" {} ", disk.sys_device))
            .title_style(Style::default().fg(theme_palette.info));

        let disk_block_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(disk_block.inner(area.centered_vertically(Constraint::Length(5))));
        let [table, uuid_line, mount_opt_line] = disk_block_layout[..] else {
            unreachable!()
        };

        let disk_table = Table::new(
            [Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::styled(format!("{:<12}", "label"), label_style),
                    Span::styled(
                        disk.label.as_deref().unwrap_or_default(),
                        Style::default().fg(Color::White),
                    )
                    .bold(),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled(format!("| {:<14}", "filesystem"), label_style),
                    Span::styled(
                        disk.file_system.as_str(),
                        Style::default().fg(theme_palette.secondary),
                    ),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled(format!("| {:<14}", "block size"), label_style),
                    Span::styled(
                        disk.block_size.to_string(),
                        Style::default().fg(Color::White),
                    ),
                ])),
            ])],
            [Constraint::Min(0), Constraint::Min(0), Constraint::Min(0)],
        );
        let disk_uuid_line = Line::from(vec![
            Span::styled(format!("{:<12}", "uuid"), label_style),
            Span::styled(
                disk.uuid.as_deref().unwrap_or_default(),
                Style::default().fg(Color::White),
            ),
        ]);
        let disk_mount_opt_line = Line::from(vec![
            Span::styled(format!("{:<12}", "mount opts"), label_style),
            Span::styled(
                disk.mount_opts.to_string(),
                Style::default().fg(Color::White),
            ),
        ]);

        frame.render_widget(disk_block, area.centered_vertically(Constraint::Length(5)));
        frame.render_widget(disk_table, table);
        frame.render_widget(disk_uuid_line, uuid_line);
        frame.render_widget(disk_mount_opt_line, mount_opt_line);
    }

    fn disk_rows(disks: &[DiskData], context: &UiContext) -> Vec<Row<'static>> {
        let theme_palette = context.palette;
        let unit_format = context.unit_format;

        disks
            .iter()
            .map(|disk| {
                Row::new(vec![
                    Cell::from(disk.sys_device.to_string())
                        .style(Style::default().fg(theme_palette.info)),
                    Cell::from(disk.mount_point.to_string())
                        .style(Style::default().fg(Color::White)),
                    Cell::from(disk.file_system.to_string())
                        .style(Style::default().fg(theme_palette.secondary)),
                    Cell::from(disk.total_bytes.format_size(unit_format))
                        .style(Style::default().fg(Color::White)),
                    Cell::from(disk.used_bytes.format_size(unit_format)),
                    Cell::from((disk.free_bytes()).format_size(unit_format))
                        .style(Style::default().fg(Color::Green)),
                    Cell::from(usage_bar(disk.use_pct(), 20)),
                ])
            })
            .collect()
    }
}

impl Default for PoolOverview {
    fn default() -> Self {
        let mut table_state = TableState::default();
        table_state.select_first();

        Self {
            disks_selector_state: table_state,
        }
    }
}

impl Dispatcher for PoolOverview {
    fn dispatch(&mut self, action: &Action, _state: &mut AppState) -> Option<Error> {
        match action {
            Action::NextItem => self.disks_selector_state.select_next(),
            Action::PrevItem => self.disks_selector_state.select_previous(),
            _ => (),
        }
        None
    }
}
