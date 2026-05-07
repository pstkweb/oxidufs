use crate::data::pool::PoolData;

#[derive(Default)]
pub struct PoolStats {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub used_pct: u8,
}

impl PoolStats {
    pub fn from_pool(pool: &PoolData) -> Option<Self> {
        let total: u64 = pool.disks.iter().map(|d| d.total_bytes).sum();

        if total == 0 {
            return None;
        }

        let used: u64 = pool.disks.iter().map(|d| d.used_bytes).sum();

        Some(Self {
            total_bytes: total,
            used_bytes: used,
            free_bytes: total - used,
            used_pct: ((used * 100) / total) as u8,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::pool::DiskData;
    use crate::model::mount_options::MountOptions;

    fn disk(total: u64, used: u64) -> DiskData {
        DiskData {
            sys_device: "/dev/sda1".into(),
            mount_point: "/mnt/disk".into(),
            file_system: "ext4".into(),
            total_bytes: total,
            used_bytes: used,
            label: None,
            block_size: 4096,
            uuid: None,
            mount_opts: MountOptions::default(),
        }
    }

    fn pool(disks: Vec<DiskData>) -> PoolData {
        PoolData {
            mount_point: "/mnt/pool".into(),
            policy: "mfs".into(),
            fuse_options: MountOptions::default(),
            disks,
        }
    }

    #[test]
    fn from_pool_returns_none_when_no_disks() {
        let p = pool(vec![]);

        assert!(PoolStats::from_pool(&p).is_none());
    }

    #[test]
    fn from_pool_returns_none_when_total_is_zero() {
        let p = pool(vec![disk(0, 0)]);

        assert!(PoolStats::from_pool(&p).is_none());
    }

    #[test]
    fn from_pool_sums_total_used_and_free() {
        let p = pool(vec![disk(100, 30), disk(200, 50)]);
        let stats = PoolStats::from_pool(&p).unwrap();

        assert_eq!(stats.total_bytes, 300);
        assert_eq!(stats.used_bytes, 80);
        assert_eq!(stats.free_bytes, 220);
    }

    #[test]
    fn from_pool_computes_used_percent() {
        let p = pool(vec![disk(1000, 250)]);
        let stats = PoolStats::from_pool(&p).unwrap();

        assert_eq!(stats.used_pct, 25);
    }

    #[test]
    fn default_is_all_zeroes() {
        let stats = PoolStats::default();

        assert_eq!(stats.total_bytes, 0);
        assert_eq!(stats.used_bytes, 0);
        assert_eq!(stats.free_bytes, 0);
        assert_eq!(stats.used_pct, 0);
    }
}
