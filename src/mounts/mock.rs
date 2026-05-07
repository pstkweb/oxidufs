use std::{fs, path::Path};

use crate::{
    model::mount_options::MountOptions,
    mounts::{MergerfsMount, MountEntry, MountSource},
};

struct ParsedMountLine<'a> {
    mount_point: &'a str,
    fs_type: &'a str,
    source: &'a str,
    super_opts: &'a str,
    mount_opts: &'a str,
}

pub struct FixtureMountSource {
    content: String,
}

impl FixtureMountSource {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            content: fs::read_to_string(path)?,
        })
    }
}

impl MountSource for FixtureMountSource {
    fn mergerfs_mounts(&self, _verbose: bool) -> anyhow::Result<Vec<MergerfsMount>> {
        parse_mergerfs_mounts(mountinfo_section(&self.content))
    }

    fn all_mounts(&self) -> anyhow::Result<Vec<MountEntry>> {
        parse_all_mounts(mountinfo_section(&self.content))
    }
}

fn mountinfo_section(content: &str) -> &str {
    content.split("---disks---").next().unwrap_or(content)
}

fn non_empty_lines(content: &str) -> impl Iterator<Item = &str> {
    content.lines().map(str::trim).filter(|l| !l.is_empty())
}

fn parse_mergerfs_mounts(content: &str) -> anyhow::Result<Vec<MergerfsMount>> {
    non_empty_lines(content)
        .map(parse_mergerfs_line)
        .filter_map(|res| match res {
            Ok(Some(m)) => Some(Ok(m)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        })
        .collect()
}

fn parse_all_mounts(content: &str) -> anyhow::Result<Vec<MountEntry>> {
    non_empty_lines(content).map(parse_mount_entry).collect()
}

fn parse_mount_line(line: &str) -> anyhow::Result<ParsedMountLine<'_>> {
    let (before_sep, after_sep) = line
        .split_once(" - ")
        .ok_or_else(|| anyhow::anyhow!("Invalid line (no '-' separator): {line}"))?;

    // BEFORE
    let mut before_parts = before_sep.split(' ');
    let _ = before_parts.next();
    let _ = before_parts.next();
    let _ = before_parts.next();
    let _ = before_parts.next();

    let mount_point = before_parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing mount point"))?;

    let mount_opts = before_parts.next().unwrap_or("");

    // AFTER
    let mut after_parts = after_sep.splitn(3, ' ');
    let fs_type = after_parts.next().unwrap_or("").trim();
    let source = after_parts.next().unwrap_or("").trim();
    let super_opts = after_parts.next().unwrap_or("").trim();

    Ok(ParsedMountLine {
        mount_point,
        fs_type,
        source,
        super_opts,
        mount_opts,
    })
}

fn parse_mergerfs_line(line: &str) -> anyhow::Result<Option<MergerfsMount>> {
    let parsed = parse_mount_line(line)?;

    if parsed.fs_type != "fuse.mergerfs" {
        return Ok(None);
    }

    let options = MountOptions::from(parsed.super_opts);

    let branches = options
        .get("branches")
        .and_then(|b| b.as_deref())
        .map(|b| {
            b.split(':')
                .map(|p| unescape_octal(p.trim()))
                .filter(|p| !p.is_empty())
                .collect()
        })
        .unwrap_or_default();

    Ok(Some(MergerfsMount {
        mount_point: unescape_octal(parsed.mount_point),
        branches,
        options,
    }))
}

fn parse_mount_entry(line: &str) -> anyhow::Result<MountEntry> {
    let parsed = parse_mount_line(line)?;

    Ok(MountEntry {
        mount_point: unescape_octal(parsed.mount_point),
        device: unescape_octal(parsed.source),
        fs_type: parsed.fs_type.to_string(),
        mount_options: MountOptions::from(parsed.mount_opts),
    })
}

fn unescape_octal(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            let octal: String = chars.by_ref().take(3).collect();
            if let Ok(n) = u8::from_str_radix(&octal, 8) {
                result.push(n as char);
            } else {
                result.push('\\');
                result.push_str(&octal);
            }
        } else {
            result.push(c);
        }
    }

    result
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
    fn single_pool_fixture_has_three_branches() {
        let src = FixtureMountSource::from_file(&fixture("mountinfo_single_pool")).unwrap();
        let pools = src.mergerfs_mounts(false).unwrap();

        assert_eq!(pools.len(), 1);
        assert_eq!(pools[0].mount_point, "/mnt/pool");
        assert_eq!(
            pools[0].branches,
            vec!["/mnt/disk1", "/mnt/disk2", "/mnt/disk3"],
        );
    }

    #[test]
    fn dual_pools_fixture_returns_two_mounts() {
        let src = FixtureMountSource::from_file(&fixture("mountinfo_dual_pools")).unwrap();
        let pools = src.mergerfs_mounts(false).unwrap();

        assert_eq!(pools.len(), 2);
        let mounts: Vec<&str> = pools.iter().map(|p| p.mount_point.as_str()).collect();
        assert!(mounts.contains(&"/mnt/pool"));
        assert!(mounts.contains(&"/mnt/pool2"));
    }

    #[test]
    fn empty_pool_fixture_has_no_branches() {
        let src = FixtureMountSource::from_file(&fixture("mountinfo_empty_pool")).unwrap();
        let pools = src.mergerfs_mounts(false).unwrap();

        assert_eq!(pools.len(), 1);
        assert!(pools[0].branches.is_empty());
    }

    #[test]
    fn mergerfs_options_are_parsed_from_super_options() {
        let src = FixtureMountSource::from_file(&fixture("mountinfo_single_pool")).unwrap();
        let pools = src.mergerfs_mounts(false).unwrap();

        assert_eq!(
            pools[0].options.get("category.create"),
            Some(&Some("mfs".to_string())),
        );
        assert_eq!(pools[0].options.get("allow_other"), Some(&None));
    }

    #[test]
    fn all_mounts_includes_underlying_disks_and_pool() {
        let src = FixtureMountSource::from_file(&fixture("mountinfo_single_pool")).unwrap();
        let entries = src.all_mounts().unwrap();

        let mounts: Vec<&str> = entries.iter().map(|e| e.mount_point.as_str()).collect();
        assert!(mounts.contains(&"/mnt/disk1"));
        assert!(mounts.contains(&"/mnt/disk2"));
        assert!(mounts.contains(&"/mnt/disk3"));
        assert!(mounts.contains(&"/mnt/pool"));
    }

    #[test]
    fn from_file_errors_when_path_does_not_exist() {
        let result = FixtureMountSource::from_file(&fixture("does_not_exist"));
        assert!(result.is_err());
    }
}
