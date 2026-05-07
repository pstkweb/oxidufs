use std::io::Write;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::{NamedTempFile, TempDir};

// ---------- Helpers -------------------------------------------------------

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Spawn the binary with a clean env so user / CI runner state doesn't leak.
fn oxidufs() -> Command {
    let mut cmd = Command::cargo_bin("oxidufs").unwrap();
    cmd.env_remove("OXIDUFS_FIXTURE")
        .env_remove("OXIDUFS_MOUNT")
        .env_remove("OXIDUFS_UNIT")
        .env_remove("OXIDUFS_THEME")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("HOME")
        .env_remove("NO_COLOR");
    cmd
}

/// Build a mountinfo fixture without any `fuse.mergerfs` line.
fn fixture_no_mergerfs() -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(
        f,
        "36 25 0:33 / /mnt/disk1 rw,relatime shared:18 - ext4 /dev/sda1 rw"
    )
    .unwrap();
    writeln!(f, "---disks---").unwrap();
    f
}

/// Create an XDG_CONFIG_HOME-compatible tempdir containing `oxidufs/config.toml`.
fn xdg_with_config(content: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    let oxi = dir.path().join("oxidufs");
    std::fs::create_dir_all(&oxi).unwrap();
    std::fs::write(oxi.join("config.toml"), content).unwrap();
    dir
}

// ---------- Output formats ------------------------------------------------

#[test]
fn plain_single_pool_renders_table() {
    oxidufs()
        .env("OXIDUFS_FIXTURE", fixture("mountinfo_single_pool"))
        .arg("--non-interactive")
        .assert()
        .success()
        .stdout(predicate::str::contains("Pool:"))
        .stdout(predicate::str::contains("Policy:"))
        .stdout(predicate::str::contains("/mnt/pool"))
        .stdout(predicate::str::contains("/dev/sda1"))
        .stdout(predicate::str::contains("/dev/sdb1"))
        .stdout(predicate::str::contains("/dev/sdc1"));
}

#[test]
fn json_output_is_parseable_and_matches_schema() {
    let out = oxidufs()
        .env("OXIDUFS_FIXTURE", fixture("mountinfo_single_pool"))
        .arg("--json")
        .output()
        .unwrap();

    assert!(out.status.success(), "binary should exit 0");

    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout must be parseable JSON");

    assert_eq!(v["pool"]["mount"], "/mnt/pool");
    assert_eq!(v["pool"]["policy"], "mfs");
    assert_eq!(v["disks"].as_array().unwrap().len(), 3);
    assert_eq!(v["disks"][0]["device"], "/dev/sda1");
    assert_eq!(v["disks"][0]["fs"], "ext4");
}

// ---------- Auto-detection / mount resolution -----------------------------

#[test]
fn multiple_pools_no_arg_lists_and_exits_2() {
    oxidufs()
        .env("OXIDUFS_FIXTURE", fixture("mountinfo_dual_pools"))
        .arg("--non-interactive")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("multiple mergerfs pools"))
        .stderr(predicate::str::contains("/mnt/pool"))
        .stderr(predicate::str::contains("/mnt/pool2"));
}

#[test]
fn no_mergerfs_mounts_errors_with_exit_2() {
    let f = fixture_no_mergerfs();

    oxidufs()
        .env("OXIDUFS_FIXTURE", f.path())
        .arg("--non-interactive")
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "no mergerfs mount points detected",
        ));
}

#[test]
fn invalid_mount_arg_exits_2() {
    oxidufs()
        .env("OXIDUFS_FIXTURE", fixture("mountinfo_single_pool"))
        .args(["--non-interactive", "/mnt/typo"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "/mnt/typo is not a mergerfs mount point",
        ));
}

// ---------- Config cascade ------------------------------------------------

#[test]
fn config_unit_used_when_no_flag() {
    let xdg = xdg_with_config(
        r#"
[display]
unit = "binary"
"#,
    );

    oxidufs()
        .env("XDG_CONFIG_HOME", xdg.path())
        .env("OXIDUFS_FIXTURE", fixture("mountinfo_single_pool"))
        .arg("--non-interactive")
        .assert()
        .success()
        // binary mode → TiB suffix appears
        .stdout(predicate::str::contains("TiB"));
}

#[test]
fn flag_unit_wins_over_config_unit() {
    let xdg = xdg_with_config(
        r#"
[display]
unit = "binary"
"#,
    );

    oxidufs()
        .env("XDG_CONFIG_HOME", xdg.path())
        .env("OXIDUFS_FIXTURE", fixture("mountinfo_single_pool"))
        .args(["--non-interactive", "-u", "decimal"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TB"))
        .stdout(predicate::str::contains("TiB").not());
}

#[test]
fn config_mount_used_when_no_arg() {
    let xdg = xdg_with_config(
        r#"
[defaults]
mount = "/mnt/pool"
"#,
    );

    // Dual pools fixture would normally trigger the "multiple pools" error;
    // with config mount set, it auto-targets that pool instead.
    oxidufs()
        .env("XDG_CONFIG_HOME", xdg.path())
        .env("OXIDUFS_FIXTURE", fixture("mountinfo_dual_pools"))
        .arg("--non-interactive")
        .assert()
        .success()
        .stdout(predicate::str::contains("/mnt/pool"));
}

#[test]
fn flag_mount_arg_wins_over_config_mount() {
    let xdg = xdg_with_config(
        r#"
[defaults]
mount = "/mnt/pool"
"#,
    );

    oxidufs()
        .env("XDG_CONFIG_HOME", xdg.path())
        .env("OXIDUFS_FIXTURE", fixture("mountinfo_dual_pools"))
        .args(["--non-interactive", "/mnt/pool2"])
        .assert()
        .success()
        // The JSON-less plain output names the pool in the "Pool:" line.
        .stdout(predicate::str::contains("/mnt/pool2"));
}

#[test]
fn malformed_config_errors_clearly() {
    let xdg = xdg_with_config("[display\nunit = ");

    oxidufs()
        .env("XDG_CONFIG_HOME", xdg.path())
        .env("OXIDUFS_FIXTURE", fixture("mountinfo_single_pool"))
        .arg("--non-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid config"));
}

// ---------- Diagnostic / verbose ------------------------------------------

#[test]
fn verbose_does_not_corrupt_json_stdout() {
    let out = oxidufs()
        .env("OXIDUFS_FIXTURE", fixture("mountinfo_single_pool"))
        .args(["--json", "--verbose"])
        .output()
        .unwrap();

    assert!(out.status.success());

    // stdout must remain pure JSON.
    let _: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout must be pure JSON");

    // stderr carries the verbose info line.
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("info: auto-detected single pool"),
        "expected verbose info on stderr, got: {stderr}",
    );
}

// ---------- Argument validation -------------------------------------------

#[test]
fn bad_unit_value_errors() {
    oxidufs()
        .env("OXIDUFS_FIXTURE", fixture("mountinfo_single_pool"))
        .args(["--non-interactive", "--unit", "bananas"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn help_flag_prints_usage_and_exits_zero() {
    oxidufs()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"))
        .stdout(predicate::str::contains("--unit"))
        .stdout(predicate::str::contains("--theme"));
}

#[test]
fn version_flag_prints_version_and_exits_zero() {
    oxidufs()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}
