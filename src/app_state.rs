use crate::{
    disks::DiskSource,
    mounts::{MergerfsMount, MountSource},
};

pub struct AppState {
    pub pools: Vec<MergerfsMount>,
    pub mount_source: Box<dyn MountSource>,
    pub disk_source: Box<dyn DiskSource>,
    pub current_pool: Option<String>,
    pub verbose: bool,
}

impl AppState {
    pub fn new(
        mount_source: Box<dyn MountSource>,
        disk_source: Box<dyn DiskSource>,
        verbose: bool,
    ) -> anyhow::Result<Self> {
        let pools = mount_source.mergerfs_mounts(verbose)?;

        Ok(Self {
            pools,
            current_pool: None,
            mount_source,
            disk_source,
            verbose,
        })
    }

    pub fn set_current_pool(&mut self, mount_point: String) {
        self.current_pool = Some(mount_point);
    }

    pub fn refresh(&mut self) -> anyhow::Result<()> {
        self.pools = self.mount_source.mergerfs_mounts(self.verbose)?;
        self.disk_source.refresh(self.mount_source.as_ref())?;

        if let Some(ref pool) = self.current_pool
            && !self.pools.iter().any(|p| p.mount_point == *pool)
        {
            self.current_pool = None;
        }

        Ok(())
    }

    pub fn fs_type_at(&self, path: &str) -> Option<String> {
        self.mount_source
            .all_mounts()
            .ok()?
            .into_iter()
            .find(|m| m.mount_point == path)
            .map(|m| m.fs_type)
    }
}
