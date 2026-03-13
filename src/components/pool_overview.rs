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

use crate::theme::usage_color;
use crate::{
    app_context,
    components::{Dispatcher, Renderable},
    keymap::Action,
};

pub struct PoolOverview {
    mount_point: String,
    policy: String,
    fuse_mount_opts: Vec<String>,
    disks: Vec<Disk>,
    disks_selector_state: TableState,
}

#[derive(Clone)]
struct Disk {
    sys_device: String,
    mount_point: String,
    file_system: String,
    total_size: u64,
    used_size: u64,
    label: String,
    block_size: u16,
    uuid: String,
    mount_opts: Vec<String>,
}

struct PoolStats {
    total_size: u64,
    used_size: u64,
    free_size: u64,
    used_pct: u8,
}

struct UiContext {
    palette: ThemePalette,
    unit_format: FormatSizeOptions,
    label_style: Style,
}

impl UiContext {
    fn new() -> Self {
        Self {
            palette: app_context::theme().palette(),
            unit_format: app_context::size_format().format_options(),
            label_style: Style::default().fg(app_context::theme().palette().muted),
        }
    }
}

impl PoolOverview {
    fn render_pool_summary(&self, frame: &mut Frame, area: Rect, context: &UiContext) {
        let stats = self.pool_stats();
        let theme_palette = context.palette;
        let unit_format = context.unit_format;
        let label_style = context.label_style;

        let pool_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" Pool ")
            .title_style(Style::default().fg(theme_palette.info));

        let pool_block_layout = Layout::vertical([Constraint::Length(2), Constraint::Length(1)])
            .split(pool_block.inner(area.centered_vertically(Constraint::Length(5))));

        let pool_table = Table::new(
            [
                Row::new(vec![
                    Cell::from(Line::from(vec![
                        Span::styled(format!("{:<8}", "mount"), label_style),
                        Span::styled(
                            self.mount_point.as_str(),
                            Style::default().fg(theme_palette.info),
                        ),
                    ])),
                    Cell::from(Line::from(vec![
                        Span::styled(format!("| {:<10}", "policy"), label_style),
                        Span::styled(
                            self.policy.as_str(),
                            Style::default().fg(theme_palette.warning),
                        ),
                    ])),
                    Cell::from(Line::from(vec![
                        Span::styled(format!("| {:<10}", "disks"), label_style),
                        Span::styled(
                            self.disks.len().to_string(),
                            Style::default().fg(Color::White),
                        ),
                    ])),
                ]),
                Row::new(vec![
                    Cell::from(Line::from(vec![
                        Span::styled(format!("{:<8}", "total"), label_style),
                        Span::styled(
                            stats.total_size.format_size(unit_format),
                            Style::default().fg(Color::White),
                        ),
                    ])),
                    Cell::from(Line::from(vec![
                        Span::styled(format!("| {:<10}", "used"), label_style),
                        Span::styled(
                            stats.used_size.format_size(unit_format),
                            Style::default().fg(theme_palette.warning),
                        ),
                    ])),
                    Cell::from(Line::from(vec![
                        Span::styled(format!("| {:<10}", "free"), label_style),
                        Span::styled(
                            stats.free_size.format_size(unit_format),
                            Style::default().fg(theme_palette.success),
                        ),
                    ])),
                ]),
            ],
            [Constraint::Min(0), Constraint::Min(0), Constraint::Min(0)],
        );
        let pool_fuse_line = Line::from(vec![
            Span::styled(format!("{:<8}", "fuse"), label_style),
            Span::styled(
                self.fuse_mount_opts.join(", "),
                Style::default().fg(Color::White),
            ),
        ]);

        frame.render_widget(pool_block, area.centered_vertically(Constraint::Length(5)));
        frame.render_widget(pool_table, pool_block_layout[0]);
        frame.render_widget(pool_fuse_line, pool_block_layout[1]);
    }

    fn render_disks_table(&mut self, frame: &mut Frame, area: Rect, context: &UiContext) {
        let rows = Self::disk_rows(&self.disks, context);
        let stats = self.pool_stats();
        let theme_palette = context.palette;
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
                Cell::from(stats.total_size.format_size(unit_format))
                    .style(Style::default().fg(Color::White)),
                Cell::from(stats.used_size.format_size(unit_format))
                    .style(Style::default().fg(usage_color(stats.used_pct))),
                Cell::from(stats.free_size.format_size(unit_format))
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

    fn render_disk_details(&self, frame: &mut Frame, area: Rect, context: &UiContext) {
        let theme_palette = context.palette;
        let label_style = context.label_style;
        let disk = self.selected_disk();

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

        let disk_table = Table::new(
            [Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::styled(format!("{:<12}", "label"), label_style),
                    Span::styled(disk.label.as_str(), Style::default().fg(Color::White)).bold(),
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
            Span::styled(disk.uuid.as_str(), Style::default().fg(Color::White)),
        ]);
        let current_disk_opts: Vec<String> = disk
            .mount_opts
            .iter()
            .map(|opt| format!("[{}]", opt))
            .collect();
        let disk_mount_opt_line = Line::from(vec![
            Span::styled(format!("{:<12}", "mount opts"), label_style),
            Span::styled(
                current_disk_opts.join(" "),
                Style::default().fg(Color::White),
            ),
        ]);

        frame.render_widget(disk_block, area.centered_vertically(Constraint::Length(5)));
        frame.render_widget(disk_table, disk_block_layout[0]);
        frame.render_widget(disk_uuid_line, disk_block_layout[1]);
        frame.render_widget(disk_mount_opt_line, disk_block_layout[2]);
    }

    fn pool_stats(&self) -> PoolStats {
        let pool_size: u64 = self.disks.iter().map(|disk| disk.total_size).sum();
        let pool_used_size: u64 = self.disks.iter().map(|disk| disk.used_size).sum();
        let pool_used_pct: u8 = ((pool_used_size * 100) / pool_size) as u8;

        PoolStats {
            total_size: pool_size,
            used_size: pool_used_size,
            free_size: pool_size - pool_used_size,
            used_pct: pool_used_pct,
        }
    }

    fn disk_rows(disks: &[Disk], context: &UiContext) -> Vec<Row<'static>> {
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
                    Cell::from(disk.total_size.format_size(unit_format))
                        .style(Style::default().fg(Color::White)),
                    Cell::from(disk.used_size.format_size(unit_format)),
                    Cell::from((disk.total_size - disk.used_size).format_size(unit_format))
                        .style(Style::default().fg(Color::Green)),
                    Cell::from(usage_bar(
                        (disk.used_size * 100 / disk.total_size) as u8,
                        20,
                    )),
                ])
            })
            .collect()
    }

    fn selected_disk(&self) -> &Disk {
        &self.disks[self.disks_selector_state.selected().unwrap_or(0)]
    }
}

impl Default for PoolOverview {
    fn default() -> Self {
        let mut table_state = TableState::default();
        table_state.select_first();

        Self {
            mount_point: String::from("/mnt/pool"),
            policy: String::from("mfs"),
            fuse_mount_opts: vec![String::from("allow_other"), String::from("cache.files=off")],
            disks: vec![
                Disk {
                    sys_device: String::from("/dev/sda1"),
                    mount_point: String::from("/mnt/disk1"),
                    file_system: String::from("ext4"),
                    total_size: 8_000_000_000_000,
                    used_size: 5_200_000_000_000,
                    label: String::from("media-1"),
                    block_size: 4096,
                    uuid: String::from("a1b2c3d4-e5f6-7890-abcd-ef1234567890"),
                    mount_opts: vec![
                        String::from("rw"),
                        String::from("relatime"),
                        String::from("data=ordered"),
                        String::from("errors=remount-ro"),
                    ],
                },
                Disk {
                    sys_device: String::from("/dev/sdb1"),
                    mount_point: String::from("/mnt/disk2"),
                    file_system: String::from("xfs"),
                    total_size: 8_000_000_000_000,
                    used_size: 3_100_000_000_000,
                    label: String::from("media-2"),
                    block_size: 4096,
                    uuid: String::from("a1b2c3d4-e5f6-7890-edfg-ef1234567890"),
                    mount_opts: vec![
                        String::from("rw"),
                        String::from("relatime"),
                        String::from("data=ordered"),
                        String::from("errors=remount-ro"),
                    ],
                },
            ],
            disks_selector_state: table_state,
        }
    }
}

impl Renderable for PoolOverview {
    fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        let context = UiContext::new();
        let chunks = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Min(3),
            Constraint::Fill(1),
        ])
        .split(area);

        self.render_pool_summary(frame, chunks[0], &context);
        self.render_disks_table(frame, chunks[1], &context);
        self.render_disk_details(frame, chunks[2], &context);
    }
}

impl Dispatcher for PoolOverview {
    fn dispatch(&mut self, action: &Action) {
        match action {
            Action::NextDisk => self.disks_selector_state.select_next(),
            Action::PrevDisk => self.disks_selector_state.select_previous(),
            _ => {}
        }
    }
}

fn usage_bar(pct: u8, width: usize) -> Line<'static> {
    let filled = (width * pct as usize) / 100;
    let empty = width - filled;
    let color = usage_color(pct);

    Line::from(vec![
        Span::styled(
            symbols::shade::FULL.repeat(filled),
            Style::default().fg(color),
        ),
        Span::styled(
            symbols::shade::LIGHT.repeat(empty),
            Style::default().fg(color),
        ),
        Span::raw(format!(" {:>3}%", pct)),
    ])
}
