use humansize::{BINARY, DECIMAL, FormatSizeOptions};
use ratatui_themes::Theme;
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone, Copy)]
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

pub struct AppContext {
    pub theme: Theme,
    pub unit: UnitMode,
}

static APP_CONTEXT: OnceLock<RwLock<AppContext>> = OnceLock::new();

pub fn init(ctx: AppContext) {
    let _ = APP_CONTEXT.set(RwLock::new(ctx));
}

pub fn get() -> RwLockReadGuard<'static, AppContext> {
    APP_CONTEXT
        .get()
        .expect("AppContext not initialized")
        .read()
        .expect("AppContext poisoned")
}

pub fn theme() -> Theme {
    return get().theme;
}

pub fn size_format() -> UnitMode {
    return get().unit;
}

pub fn write() -> RwLockWriteGuard<'static, AppContext> {
    APP_CONTEXT
        .get()
        .expect("AppContext not initialized")
        .write()
        .expect("AppContext poisoned")
}
