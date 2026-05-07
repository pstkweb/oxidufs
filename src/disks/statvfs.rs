use std::collections::HashMap;

use nix::sys::statvfs::statvfs;

use crate::{
    disks::{
        DiskSource,
        label_uuid::{device_labels, device_uuids},
    },
    model::mount_options::MountOptions,
    mounts::{MountEntry, MountSource},
};

use super::DiskInfo;

pub struct RealDiskSource {
    mount_entries: Vec<MountEntry>,
    labels: HashMap<String, String>,
    uuids: HashMap<String, String>,
}

impl RealDiskSource {
    pub fn new(mount_source: &dyn MountSource) -> anyhow::Result<Self> {
        Ok(Self {
            mount_entries: mount_source.all_mounts()?,
            labels: device_labels(),
            uuids: device_uuids(),
        })
    }
}

impl DiskSource for RealDiskSource {
    fn refresh(&mut self, mount_source: &dyn MountSource) -> anyhow::Result<()> {
        self.mount_entries = mount_source.all_mounts()?;
        self.labels = device_labels();
        self.uuids = device_uuids();

        Ok(())
    }

    fn disk_info(&self, mount_point: &str) -> anyhow::Result<DiskInfo> {
        let entry = self
            .mount_entries
            .iter()
            .find(|e| e.mount_point == mount_point);
        let stat = statvfs(mount_point)
            .map_err(|e| anyhow::anyhow!("statvfs({mount_point}) failed: {e}"))?;

        let (device, fs_type, mount_options) = entry
            .map(|e| (e.device.clone(), e.fs_type.clone(), e.mount_options.clone()))
            .unwrap_or_else(|| ("unknown".into(), "unknown".into(), MountOptions::default()));
        let label = self.labels.get(&device).cloned();
        let uuid = self.uuids.get(&device).cloned();

        Ok(DiskInfo {
            mount_point: mount_point.to_string(),
            device,
            fs_type,
            label,
            uuid,
            block_size: stat.block_size(),
            fragment_size: stat.fragment_size(),
            blocks: stat.blocks(),
            blocks_free: stat.blocks_free(),
            mount_options,
        })
    }
}
