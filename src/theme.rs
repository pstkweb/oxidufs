use clap::ValueEnum;
use ratatui::{
    style::Color,
    symbols,
    text::{Line, Span},
};
use ratatui_themes::{Style, ThemeName};
use serde::Deserialize;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, ValueEnum, EnumIter)]
#[serde(rename_all = "kebab-case")]
pub enum SupportedTheme {
    Dracula,
    OneDarkPro,
    Catppuccin,
    Gruvbox,
    TokyoNight,
    Solarized,
    MonokaiPro,
    Everforest,
    Cyberpunk,
}

impl SupportedTheme {
    pub fn next(self) -> Self {
        let all: Vec<Self> = Self::iter().collect();
        let idx = all.iter().position(|t| *t == self).unwrap_or(0);

        all[(idx + 1) % all.len()]
    }
}

impl From<SupportedTheme> for ThemeName {
    fn from(value: SupportedTheme) -> Self {
        match value {
            SupportedTheme::Dracula => ThemeName::Dracula,
            SupportedTheme::OneDarkPro => ThemeName::OneDarkPro,
            SupportedTheme::Catppuccin => ThemeName::CatppuccinMocha,
            SupportedTheme::Gruvbox => ThemeName::GruvboxDark,
            SupportedTheme::TokyoNight => ThemeName::TokyoNight,
            SupportedTheme::Solarized => ThemeName::SolarizedDark,
            SupportedTheme::MonokaiPro => ThemeName::MonokaiPro,
            SupportedTheme::Everforest => ThemeName::Everforest,
            SupportedTheme::Cyberpunk => ThemeName::Cyberpunk,
        }
    }
}

pub fn usage_color(pct: u8) -> Color {
    match pct {
        0..=59 => Color::Green,
        60..=79 => Color::Yellow,
        _ => Color::Red,
    }
}

pub fn usage_bar(pct: u8, width: usize) -> Line<'static> {
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
