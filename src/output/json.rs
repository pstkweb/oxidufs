use std::{io, process::ExitCode};

use crate::{
    app_state::AppState,
    data::pool::{DiskData, PoolData},
    model::pool::PoolStats,
    output::errors,
};

#[derive(serde::Serialize)]
struct OutputRoot {
    pool: PoolJson,
    disks: Vec<DiskJson>,
}

#[derive(serde::Serialize)]
struct PoolJson {
    mount: String,
    policy: String,
    fuse_options: String,
    size_bytes: u64,
    used_bytes: u64,
    free_bytes: u64,
    use_pct: u8,
}

impl PoolJson {
    fn from(pool: &PoolData, stats: PoolStats) -> Self {
        PoolJson {
            mount: pool.mount_point.clone(),
            policy: pool.policy.clone(),
            fuse_options: pool.fuse_options.to_string(),
            size_bytes: stats.total_bytes,
            used_bytes: stats.used_bytes,
            free_bytes: stats.free_bytes,
            use_pct: stats.used_pct,
        }
    }
}

#[derive(serde::Serialize)]
struct DiskJson {
    device: String,
    mount: String,
    fs: String,
    label: Option<String>,
    size_bytes: u64,
    used_bytes: u64,
    free_bytes: u64,
    use_pct: u8,
    mount_options: Vec<String>,
}

impl From<&DiskData> for DiskJson {
    fn from(value: &DiskData) -> Self {
        Self {
            device: value.sys_device.clone(),
            mount: value.mount_point.clone(),
            fs: value.file_system.clone(),
            label: value.label.clone(),
            size_bytes: value.total_bytes,
            used_bytes: value.used_bytes,
            free_bytes: value.free_bytes(),
            use_pct: value.use_pct(),
            mount_options: value.mount_opts.to_string_list(),
        }
    }
}

pub fn render(mount_point: &str, state: &AppState) -> anyhow::Result<ExitCode> {
    let Some(pool_data) = PoolData::load(mount_point, state) else {
        return Ok(errors::not_a_mergerfs_mount(mount_point.to_string()));
    };

    let stats = PoolStats::from_pool(&pool_data).unwrap_or_default();

    serde_json::to_writer_pretty(
        io::stdout(),
        &OutputRoot {
            pool: PoolJson::from(&pool_data, stats),
            disks: pool_data.disks.iter().map(DiskJson::from).collect(),
        },
    )?;
    println!();

    Ok(ExitCode::SUCCESS)
}
