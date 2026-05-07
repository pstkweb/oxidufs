# OxiDuFs — Architecture Notes

Companion to `CLAUDE.md`. Documents the *why* behind design decisions so they don't get relitigated.

---

## Component pattern (Ratatui)

OxiDuFs uses Ratatui's **component pattern**, not Elm, not Flux.

Owner has a React background — this is the primary mental model used when reasoning about architecture:
- Props flow down at render time (no stored references to parent state).
- Components are composable, not monolithic.
- `AppState` is passed as a parameter to `draw()` — analogous to React props being passed at render time rather than stored in component structs. This avoids lifetime propagation issues.

## `AppState` and `UiContext`

- `AppState` is the single source of truth for runtime data.
- `UiContext` is a **single-frame cache** created at the start of each `draw()` call. It caches a snapshot of the global `AppContext` (themes, units, etc.) to avoid repeated `RwLock` acquisitions within one frame.
- The global `AppContext` lives in an `OnceLock<RwLock<AppContext>>` — this pattern was chosen explicitly and should not be replaced with thread-locals or lazy_static.

## Trait objects for data sources

```rust
// In AppState
mount_source: Box<dyn MountSource>,
disk_source: Box<dyn DiskSource>,
```

Rationale:
- Enables swapping between `procfs`-backed real implementations and fixture-backed mocks.
- Local development works without a real mergerfs mount.
- Crate upgrades only require changing the concrete type, not the call sites.

## `DiskInfo` → `DiskData` pipeline

```
statvfs syscall
    ↓
DiskInfo  (raw values: blocks, block_size, blocks_free, …)
    ↓ From<DiskInfo>
DiskData  (human-facing: size_bytes, free_bytes, use_pct as methods)
```

- `DiskInfo` is a thin wrapper around syscall output — no computed fields.
- `DiskData` is what components receive. Computed fields are methods, never stored.

## `PoolData::load()` and `PoolStats::from_pool()`

Two distinct failure modes, two distinct types:
- `PoolData::load() -> Result<PoolData>` — the pool path was not found or is not a mergerfs mount. Hard failure.
- `PoolStats::from_pool(pool: &PoolData) -> Option<PoolStats>` — pool exists but has no member disks. Soft failure; `unwrap_or_default()` with a derived `Default` produces a zeroed stats struct.

## mergerfs xattr reads

The `.mergerfs` virtual file at the pool root (e.g. `/mnt/pool/.mergerfs`) exposes all mergerfs runtime options as extended attributes. This is the only reliable way to get options like `category.create`, `category.search`, `func.create`, etc.

**Allowlist approach:** do not expose all ~70 options. Curate ~15 relevant keys for the UI:

Relevant keys (approximate):
- `category.create`, `category.search`, `category.action`
- `func.create`, `func.open`
- `cache.files`, `cache.symlinks`, `cache.statfs`
- `direct_io`, `dropcacheonclose`
- `minfreespace`
- `moveonenospc`
- `ignorepponrename`
- `security_capability`
- `xattr`

## Branch extraction fallback chain

When resolving the list of member disks (branches) for a mergerfs mount:

```
1. Parse super_options from /proc/self/mountinfo  →  "branch=..." field
2. Fall back to mount_source field in mountinfo
3. Fall back to xattr read from .mergerfs control file
```

The first strategy that yields a non-empty result is used.

## Fixture file format

```
/proc/self/mountinfo-style lines for mergerfs mounts
...
---disks---
/proc/self/mountinfo-style lines (or custom format) for member disks
...
```

Loaded when `OXIDUFS_FIXTURE` env var is set. Used for development and testing without a real mergerfs pool.

## Unit display

```rust
pub enum UnitMode {
    Decimal,  // KB, MB, GB — base 1000 — DEFAULT
    Binary,   // KiB, MiB, GiB — base 1024
}
```

`auto` and `bytes` were removed. Do not re-add them. The default is always `Decimal`.

## Refresh

Manual only. The `r` key triggers a full data reload. No timers, no background threads polling for changes. This was a deliberate simplification — automatic refresh added complexity without meaningful benefit for the target use case (homelab inspection tool, not a live monitor).
