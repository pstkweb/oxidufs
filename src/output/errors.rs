use std::process::ExitCode;

pub fn not_a_mergerfs_mount(mount_point: String) -> ExitCode {
    eprintln!("error: {} is not a mergerfs mount point", mount_point);
    eprintln!();
    eprintln!("tip: specify a mergerfs mount point with `oxidufs /mnt/pool`");

    ExitCode::from(2)
}

pub fn no_mergerfs_mount() -> ExitCode {
    eprintln!("error: no mergerfs mount points detected on this system");
    eprintln!();
    eprintln!("tip: specify a mount point explicitly with `oxidufs /mnt/pool`");
    eprintln!("tip: is mergerfs installed? try `which mergerfs`");

    ExitCode::from(2)
}

pub fn multiple_pools(pools: &[String]) -> ExitCode {
    eprintln!("error: multiple mergerfs pools detected — please specify one:");
    eprintln!();

    for pool in pools {
        eprintln!("   `oxidufs  {}`", pool);
    }

    ExitCode::from(2)
}
