# OxiDuFs — CLI UX Reference

> oxi → oxidation → Rust | du → disk usage | fs → filesystem  
> A mergerfs pool inspector built in Rust with Ratatui

---

## 1. Overview

OxiDuFs is a terminal tool for inspecting and managing mergerfs storage pools. It provides a rich TUI by default and a pipe-friendly plain text / JSON mode for scripting.

> **Core Philosophy**  
> Progressive disclosure — simple by default, powerful when you need it. The tool auto-detects your mergerfs pool and launches immediately.

### 1.1 Auto-mode Detection

OxiDuFs mirrors the convention of tools like `ls` and `grep --color=auto`:

```
stdout is a TTY?
├── YES → launch Ratatui TUI
└── NO  → plain text output (pipe-friendly)
```

### 1.2 Auto-detection Logic

When no mount point is specified:

```
No mount point given?
├── Find 1 mergerfs mount  → use it silently
├── Find 2+ mounts         → interactive picker (TUI) / error (non-interactive)
└── Find 0 mounts          → friendly error with tips
```

---

## 2. Invocation

OxiDuFs is designed to be invoked with minimal arguments:

```bash
# Auto-detect mergerfs mounts on the system
oxidufs

# Target a specific mergerfs mount point
oxidufs /mnt/pool
```

> **Note**  
> The positional argument is always a mergerfs mount point, never an underlying disk path. Pointing at a member disk will produce a clear error.

---

## 3. Flags & Options

### 3.1 Phase 1 — Core (Available Now)

**Targeting**

| Flag | Description | Default |
|------|-------------|---------|
| `<MOUNT>` | mergerfs mount point to inspect | auto-detect |

**Display**

| Flag | Description | Default |
|------|-------------|---------|
| `-u, --unit <UNIT>` | `binary` \| `decimal` | `decimal` |
| `--no-color` | Disable colors (respects `NO_COLOR` env var) | — |
| `--theme <THEME>` | `dracula` \| `one-dark-pro` \| `catppuccin` \| `gruvbox` \| `tokyo-night` \| `solarized` \| `monokai-pro` \| `everforest` \| `cyberpunk` | `catppuccin` |

**Output Mode**

| Flag | Description | Default |
|------|-------------|---------|
| `-n, --non-interactive` | Plain text output, no TUI | — |
| `-j, --json` | JSON output (implies `-n`) | — |

**Misc**

| Flag | Description | Default |
|------|-------------|---------|
| `-v, --verbose` | Show warnings, skipped paths, etc. | — |
| `-V, --version` | Print version and exit | — |
| `-h, --help` | Print help and exit | — |

### 3.2 Phase 2 — File Distribution (Planned v0.2)

| Command | Description |
|---------|-------------|
| `oxidufs where <FILE>` | Resolve the physical disk path of a file in the pool |
| `oxidufs files [PATH]` | Show file/folder distribution across member disks |

### 3.3 Phase 3 — Health & Warnings (Planned v0.3)

| Command | Description |
|---------|-------------|
| `oxidufs health` | S.M.A.R.T. status per disk |
| `--warn-threshold <N>` | Warn when any disk usage exceeds N% (e.g. 85) |

### 3.4 Phase 4 — Advanced (Planned v0.4)

| Command | Description |
|---------|-------------|
| `oxidufs rebalance --dry-run` | Preview rebalancing operations without executing |
| `oxidufs rebalance --execute` | Execute rebalancing (requires confirmation) |
| `oxidufs policy edit` | Interactively edit mergerfs policy |

> **Future Subcommands — Not Yet Implemented**  
> Running any Phase 2–4 subcommand before it is released will print a clear error:  
> `error: 'oxidufs where' is not yet available (planned for v0.2) — see: https://github.com/you/oxidufs/releases`

---

## 4. TUI vs. CLI Subcommand Map

OxiDuFs has two surfaces: the interactive TUI and the CLI subcommands. They expose the same capabilities — the CLI is for scripting and piping, the TUI is for interactive exploration.

| CLI Subcommand | TUI Equivalent | Phase |
|----------------|----------------|-------|
| `oxidufs [MOUNT]` | Launch screen — pool overview | 1 |
| `oxidufs where <FILE>` | Press `W`, type path → result inline | 2 |
| `oxidufs files [PATH]` | Tab / F2 → File Distribution panel | 2 |
| `oxidufs health` | Tab / F3 → Health panel | 3 |
| `--warn-threshold` | Inline warnings on disk rows | 3 |
| `oxidufs rebalance --dry-run` | F4 / R → Rebalance preview screen | 4 |
| `oxidufs rebalance --execute` | Confirm dialog inside rebalance screen | 4 |
| `oxidufs policy edit` | Tab / F5 → Policy editor panel | 4 |

> **Design Rule**  
> CLI subcommands are for scripting and piping. The TUI exposes the same features as interactive navigation using keys and panels. No feature is exclusive to one surface.

---

## 5. Output Formats

### 5.1 Non-interactive (Plain Text)

```
$ oxidufs /mnt/pool --non-interactive

Pool:      /mnt/pool
Policy:    mfs (most-free-space)
FUSE opts: allow_other,cache.files=off

DISK       MOUNT       FS    SIZE   USED   FREE   USE%
/dev/sda1  /mnt/disk1  ext4  8.0T   5.2T   2.8T    65%
/dev/sdb1  /mnt/disk2  ext4  8.0T   3.1T   4.9T    39%
/dev/sdc1  /mnt/disk3  xfs   12.0T  9.8T   2.2T    82% ⚠
──────────────────────────────────────────────────────────
POOL       /mnt/pool         28.0T  18.1T  9.9T    65%
```

### 5.2 JSON Output

```
$ oxidufs /mnt/pool --json
```

```json
{
  "pool": {
    "mount": "/mnt/pool",
    "policy": "mfs",
    "fuse_options": "allow_other,cache.files=off",
    "size_bytes": 30798371135488,
    "used_bytes": 19428372938752,
    "free_bytes": 10633998196736,
    "use_pct": 65
  },
  "disks": [
    {
      "device": "/dev/sda1",
      "mount": "/mnt/disk1",
      "fs": "ext4",
      "label": "media-1",
      "size_bytes": 8796093022208,
      "used_bytes": 5726623063040,
      "free_bytes": 3069469958168,
      "use_pct": 65,
      "mount_options": ["rw", "relatime"]
    }
  ]
}
```

---

## 6. Unit Size Syntax

Used across flags like `--min-size`, `--warn-threshold`, etc. The suffix determines both the scale and the base system:

| Input | Mode | Meaning |
|-------|------|---------|
| `10` | bytes | 10 bytes (no conversion) |
| `10K` | si | 10 KB = 10 000 bytes |
| `10M` | si | 10 MB = 10 000 000 bytes |
| `10G` | si | 10 GB = 10 000 000 000 bytes |
| `10Ki` | binary | 10 KiB = 10 240 bytes |
| `10Mi` | binary | 10 MiB = 10 485 760 bytes |
| `10Gi` | binary | 10 GiB = 10 737 418 240 bytes |

---

## 7. Error UX

Errors are always actionable — every error includes a tip pointing to a resolution.

**No mergerfs mount found**

```
$ oxidufs

error: no mergerfs mount points detected on this system

tip: specify a mount point explicitly with `oxidufs /mnt/pool`
tip: is mergerfs installed? try `which mergerfs`
```

**Not a mergerfs mount**

```
$ oxidufs /mnt/disk1

error: /mnt/disk1 is not a mergerfs mount point (it is ext4)

tip: point oxidufs at your pool root, not an underlying disk
```

**Multiple pools, no argument (non-interactive)**

```
$ oxidufs

error: multiple mergerfs pools detected — please specify one:

  /mnt/pool
  /mnt/backup-pool
```

**Bad flag value**

```
$ oxidufs --unit bananas

error: invalid value 'bananas' for '--unit <UNIT>'

[possible values: decimal, binary]

tip: use `oxidufs --help` for full usage
```

---

## 8. Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `OXIDUFS_MOUNT` | Default mount point | `/mnt/pool` |
| `OXIDUFS_UNIT` | Default unit display (`decimal` \| `binary`) | `binary` |
| `OXIDUFS_THEME` | Default theme | `catppuccin` |
| `OXIDUFS_THREADS` | Scan parallelism (Phase 2+, not yet implemented) | `4` |
| `NO_COLOR` | Disable all colors (standard) | `1` |

---

## 9. Config File

Location: `~/.config/oxidufs/config.toml` (lower priority than flags and env vars)

```toml
[display]
unit = "decimal"   # decimal | binary
theme = "catppuccin"

[defaults]
mount = "/mnt/pool"
```

---

## 10. Example Workflows

```bash
# Launch TUI on auto-detected pool
oxidufs

# Target a specific pool
oxidufs /mnt/pool

# Quick overview, no TUI
oxidufs /mnt/pool --non-interactive

# Export for reporting or scripting
oxidufs /mnt/pool --json > report.json

# Pipe to jq — disks over 80% usage
oxidufs /mnt/pool --json | jq '.disks[] | select(.use_pct > 80)'

# Binary units, light theme
oxidufs /mnt/pool --unit binary --theme light

# No colors (e.g. for logging)
oxidufs /mnt/pool --non-interactive --no-color
```

---

## 11. Roadmap Summary

CLI surface added per phase:

| Phase | Version | CLI Surface Added |
|-------|---------|-------------------|
| Phase 1 — Core | v0.1 | `oxidufs [MOUNT]` + display flags + `-n` + `-j` |
| Phase 2 — File Distribution | v0.2 | `oxidufs where <FILE>`, `oxidufs files [PATH]` |
| Phase 3 — Health & Warnings | v0.3 | `oxidufs health`, `--warn-threshold` |
| Phase 4 — Advanced | v0.4 | `oxidufs rebalance`, `oxidufs policy` |

---

*Built with Rust & Ratatui · oxi → oxidation → Rust · du → disk usage · fs → filesystem*
