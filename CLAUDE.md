# OxiDuFs — Claude Code Project Memory

> oxidufs — a TUI for your mergerfs universe  
> `oxi` → oxidation → Rust | `du` → disk usage | `fs` → filesystem

---

## What is this project?

OxiDuFs is a **Rust + Ratatui TUI** for inspecting and managing **mergerfs storage pools**.  
It is intentionally generic — useful to any mergerfs user, not tailored to a specific homelab setup.

**Core philosophy:** read-before-write. Informational features first, mutating operations last.

---

## Roadmap (phases)

| Phase | Version | Theme | Status |
|-------|---------|-------|--------|
| 1 | v0.1 | Core MVP — pool/disk overview, usage, attributes, mergerfs config | ✓ Done |
| 2 | v0.2 | Key differentiator — file distribution, physical path resolution | Planned |
| 3 | v0.3 | Power features — S.M.A.R.T., usage warnings, policy simulation | Planned |
| 4 | v0.4 | Advanced — rebalancing (preview + execute), policy editing | Planned |

Full details: see `docs/Roadmap.md` and `docs/CLI-UX-Reference.md`.

---

## Tech stack

- **Language:** Rust
- **TUI framework:** Ratatui (component pattern — NOT Elm/Flux/Redux)
- **Key crates:** `procfs`, `nix` (statvfs), `anyhow`, `crossterm`
- **Runtime:** Linux only (homelab / server), mergerfs filesystem

---

## Module layout

```
src/
  disks/        # statvfs, label/uuid discovery
  mounts/       # procfs-based mount parsing
  model/        # value objects, computed data (MountOptions, PoolStats, …)
  data/         # component-facing assembled models (DiskData, PoolData, …)
  components/   # Ratatui components (PoolOverview, PoolPicker, Help, StatusBar, …)
  output/       # non-interactive surfaces (plain text table, JSON, error messages)
  config.rs     # ~/.config/oxidufs/config.toml loader + flag/env/config resolution
```

---

## Architecture decisions (locked)

### Data sources
- `/proc/self/mountinfo` is preferred over `/proc/mounts` — more complete, no truncation risk.
- Read via the `procfs` crate for real implementations; manual parsing only for mock fixtures.

### Trait abstractions
- `MountSource` and `DiskSource` are **boxed trait objects** stored in `AppState`.
- This enables crate-swapping and local development mocking without changing call sites.

### Mock / fixture system
- Single fixture file with a `---disks---` separator between mounts and disks sections.
- Loaded via the `OXIDUFS_FIXTURE` env variable.

### `MountOptions`
- Newtype wrapping `HashMap<String, Option<String>>`, lives in `src/model/`.
- Has `From` implementations for `&str`, `String`, and the underlying map.

### `DiskInfo` vs `DiskData`
- `DiskInfo` stores **raw** statvfs values.
- Computed fields (`free_bytes`, `use_pct`) are **methods**, not stored fields.
- `DiskData` performs conversion via `From<DiskInfo>`.

### mergerfs options
- `category.create` and similar options are **absent** from `/proc/self/mountinfo` super_options.
- They must be read via **xattr** from the `.mergerfs` control file.
- Use a curated allowlist of ~15 relevant keys (not all ~70).

### Branch extraction
Three-strategy fallback (in order):
1. `super_options` field in mountinfo
2. `mount_source` field
3. xattr from `.mergerfs` control file

### Render architecture
- `AppState` is passed **as a parameter** to `draw()` at each frame.
- This is the React "props at render time" model — avoids lifetime propagation through component structs.
- `UiContext` is retained as a single-frame cache to avoid repeated lock acquisitions.

### Global config
- `OnceLock<RwLock<AppContext>>` for global UI config — retained by explicit design choice.

### Refresh
- **Manual only** via `r` key. Automatic refresh was removed.

---

## Coding conventions

- Prefer `Option<Self>` over `Result<Self>` when only one failure mode exists.
- Use `&[T]` over `&Vec<T>` for slice parameters.
- Use `Vec<Line<'static>>` for tips/styled text fields to support both plain strings and styled spans.
- Use `unwrap_or_default()` with derived `Default` on `PoolStats` for the no-disks case.
- `Result` from `PoolData::load()` handles "pool not found"; `Option` from `PoolStats::from_pool()` handles "pool has no disks".

### Unit display
Two modes: `decimal` (KB/MB/GB, base 1000, **default**), `binary` (KiB/MiB/GiB, base 1024).  
`auto` and `bytes` modes were explicitly removed and must not be re-introduced.

---

## CLI surface (Phase 1)

```
oxidufs [MOUNT]             # auto-detect or target pool
  -u, --unit <decimal|binary>
  --no-color
  --theme <dracula|one-dark-pro|catppuccin|gruvbox|tokyo-night|solarized|monokai-pro|everforest|cyberpunk>
  -n, --non-interactive
  -j, --json
  -v, --verbose
  -V, --version
  -h, --help
```

Auto-mode: if stdout is a TTY → launch Ratatui TUI. Otherwise → plain text (pipe-friendly).

### TUI ↔ CLI mapping (all phases)

| CLI | TUI | Phase |
|-----|-----|-------|
| `oxidufs [MOUNT]` | Launch screen — pool overview | 1 |
| `oxidufs where <FILE>` | Press `W`, type path → result inline | 2 |
| `oxidufs files [PATH]` | Tab / F2 → File Distribution panel | 2 |
| `oxidufs health` | Tab / F3 → Health panel | 3 |
| `--warn-threshold` | Inline warnings on disk rows | 3 |
| `oxidufs rebalance --dry-run` | F4 / R → Rebalance preview screen | 4 |
| `oxidufs rebalance --execute` | Confirm dialog inside rebalance screen | 4 |
| `oxidufs policy edit` | Tab / F5 → Policy editor panel | 4 |

**Design rule:** no feature is exclusive to one surface. CLI = scripting. TUI = interactive.

---

## Environment variables

| Variable | Description | Example |
|----------|-------------|---------|
| `OXIDUFS_FIXTURE` | Path to mock fixture file (dev/test) | `tests/fixtures/mountinfo_single_pool` |
| `OXIDUFS_MOUNT` | Default mount point | `/mnt/pool` |
| `OXIDUFS_UNIT` | Default unit (`decimal`\|`binary`) | `binary` |
| `OXIDUFS_THEME` | Default theme | `catppuccin` |
| `OXIDUFS_THREADS` | Scan parallelism (Phase 2+, not yet implemented) | `4` |
| `NO_COLOR` | Disable all colors (standard) | `1` |

---

## Config file

Location: `~/.config/oxidufs/config.toml` (lowest priority — flags > env vars > config file)

```toml
[display]
unit = "decimal"   # decimal | binary
theme = "catppuccin"

[defaults]
mount = "/mnt/pool"
```

---

## Key non-obvious facts

1. **mergerfs policy options are not in mountinfo** — `category.create`, `category.search`, etc. require reading xattr from the `.mergerfs` virtual control file at the pool root.
2. **`/proc/mounts` truncates long lines** — always use `/proc/self/mountinfo` via `procfs`.
3. **Fixture separator** — the mock file uses `---disks---` to split mounts from disk entries.
4. **Phase 2 is the key differentiator** — resolving the real physical path of a file in the pool is what separates OxiDuFs from just running `df`.

---

## Out of scope (by design)

- Tailored to any specific homelab — the tool targets the general mergerfs community.
- All Phase 2–4 subcommands must print a clear "not yet available" error until released.
- Do not re-introduce `auto` or `bytes` unit modes.
- Do not re-introduce automatic refresh.
