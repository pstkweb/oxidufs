use crate::{app_context::UnitMode, theme::SupportedTheme};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct DisplayConfig {
    pub unit: Option<UnitMode>,
    pub theme: Option<SupportedTheme>,
}

#[derive(Debug, Default, Deserialize)]
pub struct DefaultsConfig {
    pub mount: Option<String>,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let Some(path) = config_path() else {
            return Ok(Self::default());
        };
        match std::fs::read_to_string(&path) {
            Ok(content) => Self::from_str(&content),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(anyhow::anyhow!("failed to read {}: {e}", path.display())),
        }
    }

    pub fn from_str(content: &str) -> anyhow::Result<Self> {
        toml::from_str(content).map_err(|e| anyhow::anyhow!("invalid config: {e}"))
    }
}

fn config_path() -> Option<PathBuf> {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .ok()
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".config"))
        })?;
    Some(base.join("oxidufs").join("config.toml"))
}

pub fn resolve_unit(arg: Option<UnitMode>, config: Option<UnitMode>) -> UnitMode {
    arg.or(config).unwrap_or(UnitMode::Decimal)
}

pub fn resolve_theme(
    arg: Option<SupportedTheme>,
    config: Option<SupportedTheme>,
) -> SupportedTheme {
    arg.or(config).unwrap_or(SupportedTheme::Catppuccin)
}

pub fn resolve_mount(arg: Option<String>, config: Option<String>) -> Option<String> {
    arg.or(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_yields_default() {
        let c = Config::from_str("").unwrap();
        assert!(c.display.unit.is_none());
        assert!(c.defaults.mount.is_none());
    }

    #[test]
    fn full_config_parses_correctly() {
        let c = Config::from_str(
            r#"
            [display]
            unit = "binary"
            theme = "dracula"
            [defaults]
            mount = "/mnt/pool"
        "#,
        )
        .unwrap();
        assert_eq!(c.display.unit, Some(UnitMode::Binary));
        assert_eq!(c.defaults.mount.as_deref(), Some("/mnt/pool"));
    }

    #[test]
    fn malformed_toml_errors() {
        assert!(Config::from_str("[display\nunit = ").is_err());
    }

    #[test]
    fn partial_config_only_section_works() {
        let c = Config::from_str(
            r#"[display]
    unit = "binary"
    "#,
        )
        .unwrap();
        assert_eq!(c.display.unit, Some(UnitMode::Binary));
        assert!(c.defaults.mount.is_none());
    }

    #[test]
    fn resolve_unit_flag_wins_over_config() {
        assert_eq!(
            resolve_unit(Some(UnitMode::Binary), Some(UnitMode::Decimal)),
            UnitMode::Binary,
        );
    }

    #[test]
    fn resolve_unit_uses_config_when_no_flag() {
        assert_eq!(resolve_unit(None, Some(UnitMode::Binary)), UnitMode::Binary,);
    }

    #[test]
    fn resolve_unit_falls_back_to_default() {
        assert_eq!(resolve_unit(None, None), UnitMode::Decimal);
    }

    #[test]
    fn resolve_theme_cascade() {
        assert_eq!(
            resolve_theme(Some(SupportedTheme::Dracula), Some(SupportedTheme::Gruvbox)),
            SupportedTheme::Dracula,
        );
        assert_eq!(
            resolve_theme(None, Some(SupportedTheme::Gruvbox)),
            SupportedTheme::Gruvbox,
        );
        assert_eq!(resolve_theme(None, None), SupportedTheme::Catppuccin);
    }

    #[test]
    fn resolve_mount_flag_wins_over_config() {
        assert_eq!(
            resolve_mount(Some("/mnt/a".to_string()), Some("/mnt/b".to_string())),
            Some("/mnt/a".to_string()),
        );
    }

    #[test]
    fn resolve_mount_uses_config_when_no_flag() {
        assert_eq!(
            resolve_mount(None, Some("/mnt/b".to_string())),
            Some("/mnt/b".to_string()),
        );
    }

    #[test]
    fn resolve_mount_returns_none_when_neither_set() {
        assert_eq!(resolve_mount(None, None), None);
    }
}
