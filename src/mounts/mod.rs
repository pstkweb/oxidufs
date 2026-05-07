use crate::model::mount_options::MountOptions;

#[derive(Debug, Clone)]
pub struct MergerfsMount {
    pub mount_point: String,
    pub branches: Vec<String>,
    pub options: MountOptions,
}

#[derive(Debug, Clone)]
pub struct MountEntry {
    pub mount_point: String,
    pub device: String,
    pub fs_type: String,
    pub mount_options: MountOptions,
}

pub trait MountSource {
    fn mergerfs_mounts(&self, verbose: bool) -> anyhow::Result<Vec<MergerfsMount>>;
    fn all_mounts(&self) -> anyhow::Result<Vec<MountEntry>>;
}

pub mod mock;
pub mod procfs;
