use crate::model::mount_options::MountOptions;
use crate::mounts::MountSource;

pub mod label_uuid;
pub mod mock;
pub mod statvfs;

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub mount_point: String,
    pub device: String,
    pub fs_type: String,
    pub label: Option<String>,
    pub uuid: Option<String>,
    pub block_size: u64,
    pub fragment_size: u64,
    pub blocks: u64,
    pub blocks_free: u64,
    pub mount_options: MountOptions,
}

pub trait DiskSource {
    fn disk_info(&self, mount_point: &str) -> anyhow::Result<DiskInfo>;
    fn refresh(&mut self, mount_source: &dyn MountSource) -> anyhow::Result<()>;
}
