use serde::Serialize;

use crate::{app_state::AppState, disks::DiskInfo, model::mount_options::MountOptions};

#[derive(Debug, Clone, Serialize)]
pub struct PoolData {
    pub mount_point: String,
    pub policy: String,
    pub fuse_options: MountOptions,
    pub disks: Vec<DiskData>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskData {
    pub sys_device: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub label: Option<String>,
    pub block_size: u64,
    pub uuid: Option<String>,
    pub mount_opts: MountOptions,
}

impl From<DiskInfo> for DiskData {
    fn from(info: DiskInfo) -> Self {
        let total_bytes = info.blocks * info.fragment_size;
        let free_bytes = info.blocks_free * info.fragment_size;
        let used_bytes = total_bytes.saturating_sub(free_bytes);

        Self {
            sys_device: info.device,
            mount_point: info.mount_point,
            file_system: info.fs_type,
            total_bytes,
            used_bytes,
            label: info.label,
            uuid: info.uuid,
            block_size: info.block_size,
            mount_opts: info.mount_options,
        }
    }
}

impl DiskData {
    pub fn free_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.used_bytes)
    }

    pub fn use_pct(&self) -> u8 {
        if self.total_bytes == 0 {
            return 0;
        }

        ((self.used_bytes as f64 / self.total_bytes as f64) * 100.0).round() as u8
    }
}

impl PoolData {
    pub fn load(mount_point: &str, state: &AppState) -> Option<Self> {
        let pool = state.pools.iter().find(|p| p.mount_point == mount_point)?;
        let disks = pool
            .branches
            .iter()
            .filter_map(|branch| match state.disk_source.disk_info(branch) {
                Ok(info) => Some(info),
                Err(e) => {
                    if state.verbose {
                        eprintln!("warning: skipped branch {branch}: {e}");
                    }
                    None
                }
            })
            .map(DiskData::from)
            .collect();
        let policy = pool
            .options
            .get("category.create")
            .and_then(|v| v.as_deref())
            .unwrap_or("pfrd")
            .to_string();

        Some(PoolData {
            mount_point: pool.mount_point.clone(),
            policy,
            fuse_options: pool.options.clone(),
            disks,
        })
    }

    pub fn summary(&self) -> String {
        format!(
            "{} disks · {} · {}",
            self.disks.len(),
            self.mount_point,
            self.policy
        )
    }

    pub fn use_pct(&self) -> u8 {
        let total_bytes: u64 = self.disks.iter().map(|d| d.total_bytes).sum();
        let used_bytes: u64 = self.disks.iter().map(|d| d.used_bytes).sum();

        if total_bytes == 0 {
            return 0;
        }

        ((used_bytes as f64 / total_bytes as f64) * 100.0).round() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disk_info(blocks: u64, blocks_free: u64) -> DiskInfo {
        DiskInfo {
            mount_point: "/mnt/disk1".into(),
            device: "/dev/sda1".into(),
            fs_type: "ext4".into(),
            label: None,
            uuid: None,
            block_size: 4096,
            fragment_size: 4096,
            blocks,
            blocks_free,
            mount_options: MountOptions::default(),
        }
    }

    #[test]
    fn from_diskinfo_computes_total_and_used() {
        let data = DiskData::from(disk_info(100, 30));

        assert_eq!(data.total_bytes, 100 * 4096);
        assert_eq!(data.used_bytes, 70 * 4096);
    }

    #[test]
    fn free_bytes_is_total_minus_used() {
        let data = DiskData::from(disk_info(100, 30));

        assert_eq!(data.free_bytes(), 30 * 4096);
    }

    #[test]
    fn use_pct_returns_zero_when_total_is_zero() {
        let data = DiskData::from(disk_info(0, 0));

        assert_eq!(data.use_pct(), 0);
    }

    #[test]
    fn use_pct_returns_50_when_half_used() {
        let data = DiskData::from(disk_info(100, 50));

        assert_eq!(data.use_pct(), 50);
    }

    #[test]
    fn use_pct_returns_100_when_full() {
        let data = DiskData::from(disk_info(100, 0));

        assert_eq!(data.use_pct(), 100);
    }

    #[test]
    fn from_diskinfo_saturates_when_blocks_free_exceeds_blocks() {
        // pathological: blocks_free > blocks. Must not panic.
        let data = DiskData::from(disk_info(50, 100));

        assert_eq!(data.used_bytes, 0);
    }
}
