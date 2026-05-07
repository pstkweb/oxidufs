use std::path::Path;

use anyhow::Result;
use procfs::process::Process;

use crate::{model::mount_options::MountOptions, mounts::MountEntry};

use super::{MergerfsMount, MountSource};

pub struct ProcfsMountSource;

const MERGERFS_XATTR_KEYS: &[&str] = &[
    "category.create",
    "category.search",
    "category.action",
    "minfreespace",
    "moveonenospc",
    "statfs",
    "statfs_ignore",
    "cache.files",
    "cache.writeback",
    "follow-symlinks",
    "link-exdev",
    "rename-exdev",
    "version",
];

impl MountSource for ProcfsMountSource {
    fn mergerfs_mounts(&self, verbose: bool) -> Result<Vec<MergerfsMount>> {
        let mounts = Process::myself()?
            .mountinfo()?
            .into_iter()
            .filter(|m| m.fs_type == "fuse.mergerfs")
            .map(|m| {
                let branches = extract_branches(&m, verbose);
                let mut options = MountOptions::from(m.super_options);
                let mount_point = Path::new(m.mount_point.to_str().unwrap_or_default());

                enrich_with_xattrs(mount_point, &mut options);

                MergerfsMount {
                    mount_point: m.mount_point.to_string_lossy().into_owned(),
                    branches,
                    options,
                }
            })
            .collect();

        Ok(mounts)
    }

    fn all_mounts(&self) -> anyhow::Result<Vec<MountEntry>> {
        let mounts = procfs::mounts()?
            .into_iter()
            .map(|m| MountEntry {
                mount_point: m.fs_file.to_string(),
                device: m.fs_spec.to_string(),
                fs_type: m.fs_vfstype.to_string(),
                mount_options: MountOptions::from(m.fs_mntops),
            })
            .collect();

        Ok(mounts)
    }
}

fn extract_branches(mount_info: &procfs::process::MountInfo, verbose: bool) -> Vec<String> {
    // #1 strategy — super_options "branches=/mnt/disk1:/mnt/disk2" (mergerfs 3.x)
    if let Some(branches) = mount_info.super_options.get("branches")
        && let Some(paths) = branches
    {
        let candidates: Vec<String> = paths.split(':').map(String::from).collect();

        if candidates.iter().all(|p| p.starts_with('/')) {
            if verbose {
                eprintln!("info: found pool branches in 'super options' field");
            }

            return candidates;
        }
    }

    // #2 strategy — mount_source "/mnt/disk1:/mnt/disk2"
    if let Some(source) = &mount_info.mount_source {
        let candidates: Vec<String> = source.split(':').map(String::from).collect();

        if candidates.iter().all(|p| p.starts_with('/')) {
            if verbose {
                eprintln!("info: found pool branches in 'mount source' field");
            }

            return candidates;
        }
    }

    // #3 strategy — xattr on mount point
    if let Some(branches) = read_branches_xattr(&mount_info.mount_point) {
        if verbose {
            eprintln!("info: found pool branches in .mergerfs pseudo file");
        }

        return branches;
    }

    vec![]
}

fn read_branches_xattr(mount_point: &std::path::Path) -> Option<Vec<String>> {
    let control = mount_point.join(".mergerfs");
    let value = xattr::get(&control, "user.mergerfs.branches").ok()??;
    let s = std::str::from_utf8(&value).ok()?;

    let branches: Vec<String> = s
        .split(':')
        .filter_map(|entry| {
            let path = entry.split('=').next()?;
            let path = path.trim();

            if path.starts_with('/') {
                Some(path.to_string())
            } else {
                None
            }
        })
        .collect();

    if branches.is_empty() {
        None
    } else {
        Some(branches)
    }
}

fn enrich_with_xattrs(mount_point: &Path, options: &mut MountOptions) {
    let control = mount_point.join(".mergerfs");

    for key in MERGERFS_XATTR_KEYS {
        let attr = format!("user.mergerfs.{key}");

        if let Ok(Some(value)) = xattr::get(&control, attr)
            && let Ok(s) = String::from_utf8(value)
        {
            options.insert(key.to_string(), Some(s));
        }
    }
}
