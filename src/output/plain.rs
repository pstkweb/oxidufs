use std::process::ExitCode;

use humansize::FormatSize;

use crate::{
    app_context, app_state::AppState, data::pool::PoolData, model::pool::PoolStats, output::errors,
};

pub fn render(mount_point: &str, state: &AppState) -> anyhow::Result<ExitCode> {
    let Some(pool_data) = PoolData::load(mount_point, state) else {
        return Ok(errors::not_a_mergerfs_mount(mount_point.to_string()));
    };

    println!();
    print!("{:<11}", "Pool:");
    println!("{}", pool_data.mount_point);
    print!("{:<11}", "Policy:");
    println!("{}", pool_data.policy);
    print!("{:<11}", "FUSE opts:");
    println!("{}", pool_data.fuse_options);
    println!();

    print_disk_table(pool_data);

    Ok(ExitCode::SUCCESS)
}

fn print_disk_table(pool: PoolData) {
    let unit_format = app_context::size_format().format_options();

    let stats = PoolStats::from_pool(&pool).unwrap_or_default();
    let pct_strings: Vec<String> = pool
        .disks
        .iter()
        .map(|d| format!("{}%", d.use_pct()))
        .collect();
    let pool_pct_str = format!("{}%", stats.used_pct);

    let w_disk = pool
        .disks
        .iter()
        .map(|disk| disk.sys_device.len())
        .max()
        .unwrap_or(0)
        .max(4);
    let w_mount = pool
        .disks
        .iter()
        .map(|disk| disk.mount_point.len())
        .max()
        .unwrap_or(5)
        .max(pool.mount_point.len());
    let w_fs = pool
        .disks
        .iter()
        .map(|disk| disk.file_system.len())
        .max()
        .unwrap_or(0)
        .max(2);
    let w_size = pool
        .disks
        .iter()
        .map(|disk| disk.total_bytes.format_size(unit_format).len())
        .max()
        .unwrap_or(4)
        .max(stats.total_bytes.format_size(unit_format).len());
    let w_used = pool
        .disks
        .iter()
        .map(|disk| disk.used_bytes.format_size(unit_format).len())
        .max()
        .unwrap_or(4)
        .max(stats.used_bytes.format_size(unit_format).len());
    let w_free = pool
        .disks
        .iter()
        .map(|disk| disk.free_bytes().format_size(unit_format).len())
        .max()
        .unwrap_or(4)
        .max(stats.free_bytes.format_size(unit_format).len());
    let w_usep = pct_strings
        .iter()
        .map(|s| s.len())
        .max()
        .unwrap_or(0)
        .max(4)
        .max(pool_pct_str.len());
    let total_width =
        w_disk + 2 + w_mount + 2 + w_fs + 2 + w_size + 2 + w_used + 2 + w_free + 2 + w_usep;

    println!(
        "{:<w_disk$}  {:<w_mount$}  {:<w_fs$}  {:<w_size$}  {:<w_used$}  {:<w_free$}  {:<w_usep$}",
        "DISK", "MOUNT", "FS", "SIZE", "USED", "FREE", "USE%",
    );

    for (i, disk) in pool.disks.iter().enumerate() {
        println!(
            "{:<w_disk$}  {:<w_mount$}  {:<w_fs$}  {:<w_size$}  {:<w_used$}  {:<w_free$}  {:<w_usep$}",
            disk.sys_device,
            disk.mount_point,
            disk.file_system,
            disk.total_bytes.format_size(unit_format),
            disk.used_bytes.format_size(unit_format),
            disk.free_bytes().format_size(unit_format),
            &pct_strings[i],
        );
    }

    println!("{}", "─".repeat(total_width));
    println!(
        "{:<w_disk$}  {:<w_mount$}  {:<w_fs$}  {:<w_size$}  {:<w_used$}  {:<w_free$}  {:<w_usep$}",
        "POOL",
        pool.mount_point,
        "",
        stats.total_bytes.format_size(unit_format),
        stats.used_bytes.format_size(unit_format),
        stats.free_bytes.format_size(unit_format),
        pool_pct_str,
    );
}
