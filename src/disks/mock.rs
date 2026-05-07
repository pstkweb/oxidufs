use std::{collections::HashMap, fs, path::Path};

use crate::{
    disks::{DiskInfo, DiskSource},
    model::mount_options::MountOptions,
    mounts::MountSource,
};

pub struct MockDiskSource {
    disks: HashMap<String, DiskInfo>,
}

impl MockDiskSource {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let disks = parse_disk_section(&content)?;

        Ok(Self {
            disks: disks
                .into_iter()
                .map(|d| (d.mount_point.clone(), d))
                .collect(),
        })
    }
}

impl DiskSource for MockDiskSource {
    fn disk_info(&self, mount_point: &str) -> anyhow::Result<DiskInfo> {
        self.disks
            .get(mount_point)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Mock: unknown mount point {mount_point}"))
    }

    fn refresh(&mut self, _mount_source: &dyn MountSource) -> anyhow::Result<()> {
        Ok(())
    }
}

fn parse_disk_section(content: &str) -> anyhow::Result<Vec<DiskInfo>> {
    let section = content
        .split("---disks---")
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Section ---disks--- absente du fichier de fixture"))?;

    section
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(parse_disk_line)
        .collect()
}

fn parse_disk_line(line: &str) -> anyhow::Result<DiskInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 9 {
        anyhow::bail!("Ligne disque invalide (attendu 9 champs): {line}");
    }

    let mount_point = parts[0].to_string();
    let device = parts[1].to_string();
    let fs_type = parts[2].to_string();
    let label = match parts[3] {
        "-" => None,
        v => Some(v.to_string()),
    };
    let uuid = match parts[4] {
        "-" => None,
        v => Some(v.to_string()),
    };
    let block_size = parts[5].parse::<u64>()?;
    let size_bytes = parts[6].parse::<u64>()?;
    let use_pct = parts[7].parse::<u8>()?;
    let mount_options = MountOptions::from(parts[8]);

    let used_bytes = (size_bytes as f64 * use_pct as f64 / 100.0) as u64;
    let free_bytes = size_bytes.saturating_sub(used_bytes);
    let blocks = size_bytes / block_size;
    let blocks_free = free_bytes / block_size;

    Ok(DiskInfo {
        mount_point,
        device,
        fs_type,
        label,
        uuid,
        block_size,
        fragment_size: block_size,
        blocks,
        blocks_free,
        mount_options,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    #[test]
    fn disk_info_returns_disk_for_known_mount() {
        let src = MockDiskSource::from_file(&fixture("mountinfo_single_pool")).unwrap();
        let info = src.disk_info("/mnt/disk1").unwrap();

        assert_eq!(info.device, "/dev/sda1");
        assert_eq!(info.fs_type, "ext4");
        assert_eq!(info.label.as_deref(), Some("media-1"));
    }

    #[test]
    fn disk_info_errors_for_unknown_mount() {
        let src = MockDiskSource::from_file(&fixture("mountinfo_single_pool")).unwrap();
        assert!(src.disk_info("/mnt/nonexistent").is_err());
    }

    #[test]
    fn disk_info_computes_blocks_consistent_with_fixture_size() {
        let src = MockDiskSource::from_file(&fixture("mountinfo_single_pool")).unwrap();
        let info = src.disk_info("/mnt/disk1").unwrap();

        // 8 TB at 4096-byte blocks → 1_953_125_000 blocks (exact).
        assert_eq!(info.blocks * info.fragment_size, 8_000_000_000_000);
        // Fixture says 65 % used → 35 % free.
        let total = info.blocks * info.fragment_size;
        let free = info.blocks_free * info.fragment_size;
        assert_eq!((free * 100) / total, 35);
    }

    #[test]
    fn from_file_errors_when_disks_section_is_missing() {
        // The mountinfo lines exist but `---disks---` separator is required.
        let mut tmp = std::env::temp_dir();
        tmp.push("oxidufs_test_no_disks_section");
        std::fs::write(
            &tmp,
            "36 25 0:33 / /mnt/disk1 rw,relatime shared:18 - ext4 /dev/sda1 rw\n",
        )
        .unwrap();

        let result = MockDiskSource::from_file(&tmp);
        let _ = std::fs::remove_file(&tmp);

        assert!(result.is_err());
    }
}
