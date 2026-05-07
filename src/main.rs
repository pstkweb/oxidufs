use clap::Parser;
use crossterm::event::Event;
use ratatui::DefaultTerminal;
use ratatui_themes::{Theme, ThemeName};
use std::{
    io::{self, IsTerminal, Result},
    path::Path,
    process::ExitCode,
};

use app::App;
use app_context::UiConfig;
use keymap::key_to_action;

use crate::{
    app_context::UnitMode,
    app_state::AppState,
    config::Config,
    disks::{DiskSource, mock::MockDiskSource, statvfs::RealDiskSource},
    mounts::{MountSource, mock::FixtureMountSource, procfs::ProcfsMountSource},
    output::{OutputFormat, ResolvedTarget, errors},
    theme::SupportedTheme,
};

mod app;
mod app_context;
mod app_state;
mod components;
mod config;
mod data;
mod disks;
mod keymap;
mod model;
mod mounts;
mod output;
mod screen;
mod theme;

#[derive(Parser, Debug)]
#[command(
    name = "OxiDuFs",
    version = env!("CARGO_PKG_VERSION"),
    about = "Manage your mergerfs pool",
    long_about = "You will be able to view, check health and monitor your mergerfs pools.",
    after_help = concat!("Licensed under MIT. See LICENSE for details."),
)]
struct Cli {
    #[arg(env = "OXIDUFS_MOUNT")]
    mount_point: Option<String>,

    #[arg(
        short = 'u',
        long = "unit",
        value_enum,
        env = "OXIDUFS_UNIT",
        help = "Switch between decimal and binary units ; default: decimal"
    )]
    unit: Option<UnitMode>,

    #[arg(
        short = 'n',
        long = "non-interactive",
        help = "Force non interactive mode (pure CLI)"
    )]
    non_interactive: bool,

    #[arg(
        long = "theme",
        value_enum,
        env = "OXIDUFS_THEME",
        help = "Color theme to use ; default: catppuccin"
    )]
    theme: Option<SupportedTheme>,

    #[arg(
        long = "no-color",
        env = "NO_COLOR",
        help = "Disable ANSI color escape codes"
    )]
    no_color: bool,

    #[arg(
        long = "json",
        short = 'j',
        help = "Output result as JSON (silently apply --no-interactive if not provided)"
    )]
    json: bool,

    #[arg(long = "verbose", short = 'v', help = "Display some additional info")]
    verbose: bool,
}

fn main() -> anyhow::Result<ExitCode> {
    let mut args = Cli::parse();

    if args.no_color {
        // SAFETY: monothread call, before ratatui::run et any spawn.
        unsafe {
            std::env::set_var("NO_COLOR", "1");
        }
    }

    if args.json {
        args.non_interactive = true;
    }

    if !io::stdout().is_terminal() {
        args.non_interactive = true;
    }

    let (mount_source, disk_source) = build_sources()?;

    let mut state = AppState::new(
        mount_source,
        disk_source,
        args.verbose && args.non_interactive,
    )?;

    let config = Config::load()?;
    let mount_point = config::resolve_mount(args.mount_point, config.defaults.mount);
    let theme = config::resolve_theme(args.theme, config.display.theme);
    let unit = config::resolve_unit(args.unit, config.display.unit);

    app_context::init(UiConfig {
        theme,
        theme_runtime: Theme::new(ThemeName::from(theme)),
        unit,
    });

    let target = output::resolve_target(mount_point.as_deref(), &state);

    if !args.non_interactive {
        match target {
            ResolvedTarget::Pool(mount_point) => state.set_current_pool(mount_point),
            ResolvedTarget::Invalid(mount_point) => {
                errors::not_a_mergerfs_mount(mount_point);

                return Ok(ExitCode::from(2));
            }
            ResolvedTarget::Empty | ResolvedTarget::Multiple(_) => {
                // Handled by TUI
            }
        }

        ratatui::run(|terminal| render(terminal, &mut state))?;

        return Ok(ExitCode::SUCCESS);
    }

    let format = if args.json {
        OutputFormat::Json
    } else {
        OutputFormat::Plain
    };

    output::render(format, target, &state)
}

fn render(terminal: &mut DefaultTerminal, state: &mut AppState) -> Result<()> {
    let mut app = App::default();

    loop {
        app.draw(terminal, state)?;

        if let Event::Key(key) = crossterm::event::read()?
            && let Some(action) = key_to_action(key, &app.current_screen)
        {
            app.dispatch(action, state);
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}

fn build_sources() -> anyhow::Result<(Box<dyn MountSource>, Box<dyn DiskSource>)> {
    if let Ok(path) = std::env::var("OXIDUFS_FIXTURE") {
        let p = Path::new(&path);
        let mount_src = Box::new(FixtureMountSource::from_file(p)?);
        let disk_src = Box::new(MockDiskSource::from_file(p)?);

        Ok((mount_src, disk_src))
    } else {
        let mount_src = Box::new(ProcfsMountSource);
        let disk_src = Box::new(RealDiskSource::new(mount_src.as_ref())?);

        Ok((mount_src, disk_src))
    }
}
