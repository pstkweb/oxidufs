use std::process::ExitCode;

use crate::app_state::AppState;

pub mod errors;
mod json;
mod plain;

pub enum OutputFormat {
    Plain,
    Json,
}

pub enum ResolvedTarget {
    Pool(String),
    Invalid(String),
    Empty,
    Multiple(Vec<String>),
}

pub fn render(
    format: OutputFormat,
    target: ResolvedTarget,
    state: &AppState,
) -> anyhow::Result<ExitCode> {
    match (target, format) {
        (ResolvedTarget::Empty, _) => Ok(errors::no_mergerfs_mount()),
        (ResolvedTarget::Invalid(mp), _) => Ok(errors::not_a_mergerfs_mount(mp)),
        (ResolvedTarget::Multiple(pools), _) => Ok(errors::multiple_pools(&pools)),
        (ResolvedTarget::Pool(mount_point), OutputFormat::Json) => {
            json::render(&mount_point, state)
        }
        (ResolvedTarget::Pool(mount_point), OutputFormat::Plain) => {
            plain::render(&mount_point, state)
        }
    }
}

pub fn resolve_target(arg: Option<&str>, state: &AppState) -> ResolvedTarget {
    if let Some(mount_point) = arg {
        if state.pools.iter().any(|p| p.mount_point == mount_point) {
            ResolvedTarget::Pool(mount_point.to_string())
        } else {
            ResolvedTarget::Invalid(mount_point.to_string())
        }
    } else if state.pools.is_empty() {
        ResolvedTarget::Empty
    } else if state.pools.len() == 1 {
        if state.verbose {
            eprintln!(
                "info: auto-detected single pool: {}",
                state.pools[0].mount_point
            );
        }

        ResolvedTarget::Pool(state.pools[0].mount_point.clone())
    } else {
        ResolvedTarget::Multiple(
            state
                .pools
                .iter()
                .map(|pool| pool.mount_point.clone())
                .collect(),
        )
    }
}
