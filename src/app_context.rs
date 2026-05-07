use clap::ValueEnum;
use humansize::{BINARY, DECIMAL, FormatSizeOptions};
use ratatui_themes::{Theme, ThemeName};
use serde::Deserialize;
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::theme::SupportedTheme;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum UnitMode {
    Decimal,
    Binary,
}

impl UnitMode {
    pub fn toggle(&mut self) {
        *self = match self {
            UnitMode::Binary => UnitMode::Decimal,
            UnitMode::Decimal => UnitMode::Binary,
        }
    }

    pub fn format_options(self) -> FormatSizeOptions {
        match self {
            UnitMode::Binary => BINARY,
            UnitMode::Decimal => DECIMAL,
        }
    }
}

#[derive(Debug)]
pub struct UiConfig {
    pub theme: SupportedTheme,
    pub theme_runtime: Theme,
    pub unit: UnitMode,
}

impl UiConfig {
    pub fn cycle_theme(&mut self) {
        self.theme = self.theme.next();
        self.theme_runtime = Theme::new(ThemeName::from(self.theme));
    }
}

static UI_CONFIG: OnceLock<RwLock<UiConfig>> = OnceLock::new();

pub fn init(ctx: UiConfig) {
    let _ = UI_CONFIG.set(RwLock::new(ctx));
}

pub fn get() -> RwLockReadGuard<'static, UiConfig> {
    UI_CONFIG
        .get()
        .expect("AppContext not initialized")
        .read()
        .expect("AppContext poisoned")
}

pub fn theme() -> Theme {
    get().theme_runtime
}

pub fn size_format() -> UnitMode {
    get().unit
}

pub fn write() -> RwLockWriteGuard<'static, UiConfig> {
    UI_CONFIG
        .get()
        .expect("AppContext not initialized")
        .write()
        .expect("AppContext poisoned")
}
