# OxiDuFs

> A TUI for your mergerfs universe.
>
> `oxi` → oxidation → Rust · `du` → disk usage · `fs` → filesystem

OxiDuFs is a Rust + Ratatui terminal UI for inspecting [mergerfs](https://github.com/trapexit/mergerfs) storage pools. It surfaces per-disk usage, pool aggregates, mount options and mergerfs runtime options in a single screen — without leaving the terminal.

Built around a **read-before-write** philosophy: informational features ship first, mutating operations last.

---

## Status

Phase 1 (MVP) is done. See [`docs/Roadmap.md`](docs/Roadmap.md) for the full plan.

| Phase | Version | Theme | Status |
|-------|---------|-------|--------|
| 1 | v0.1 | Pool & disk overview, usage, attributes, mergerfs config | Done |
| 2 | v0.2 | File distribution, physical path resolution | Planned |
| 3 | v0.3 | S.M.A.R.T., warnings, policy simulation | Planned |
| 4 | v0.4 | Rebalance, policy editing | Planned |

---

## Requirements

- **Linux only.** OxiDuFs reads `/proc/self/mountinfo`, calls `statvfs`, and queries xattr on mergerfs control files — all Linux primitives. macOS / Windows / BSD are out of scope by design and the CI matrix only targets Linux.
- A mounted mergerfs pool (or a fixture file for dev — see below)
- Rust stable, edition 2024 (pinned in [`rust-toolchain.toml`](rust-toolchain.toml))

---

## Installation

### From a release

Grab the latest archive from [GitHub Releases](../../releases), verify the checksum, extract:

```bash
VERSION=v0.1.0
curl -LO https://github.com/pstkweb/oxidufs/releases/download/$VERSION/oxidufs-$VERSION-x86_64-linux.tar.gz
curl -LO https://github.com/pstkweb/oxidufs/releases/download/$VERSION/oxidufs-$VERSION-x86_64-linux.tar.gz.sha256
sha256sum -c oxidufs-$VERSION-x86_64-linux.tar.gz.sha256
tar -xzf oxidufs-$VERSION-x86_64-linux.tar.gz
./oxidufs --help
```

### From source

```bash
git clone https://github.com/pstkweb/oxidufs.git
cd oxidufs
cargo build --release
./target/release/oxidufs
```

---

## Usage

```bash
oxidufs                       # auto-detect mergerfs mount, launch TUI
oxidufs /mnt/pool             # target a specific pool
oxidufs --non-interactive     # plain text output
oxidufs --json | jq           # JSON output for scripting
oxidufs --help                # full flag reference
```

OxiDuFs auto-detects whether stdout is a TTY: a pipe or redirection automatically falls back to non-interactive mode. See [`docs/CLI-UX-Reference.md`](docs/CLI-UX-Reference.md) for the full CLI surface.

### Configuration

Settings can come from three sources, in priority order:

1. **CLI flags** (`-u binary`, `--theme dracula`, …)
2. **Environment variables** (`OXIDUFS_UNIT`, `OXIDUFS_THEME`, `OXIDUFS_MOUNT`, `NO_COLOR`)
3. **Config file** at `~/.config/oxidufs/config.toml` (or `$XDG_CONFIG_HOME/oxidufs/config.toml`)

```toml
[display]
unit = "binary"   # decimal | binary
theme = "catppuccin"

[defaults]
mount = "/mnt/pool"
```

A missing config file is silently treated as defaults. A malformed one fails loudly.

### Keys

| Key | Action |
|-----|--------|
| `j` / `k` / `↓` / `↑` | Move selection |
| `u` | Toggle unit (decimal / binary) |
| `t` | Cycle theme |
| `r` | Refresh |
| `?` | Toggle help popup |
| `Esc` | Close help |
| `Enter` | Pick pool (when multiple are detected) |
| `F1`–`F5` | Switch tab (`F1` Pool Overview ; `F2`–`F5` planned for Phase 2–4) |
| `q` | Quit |

---

## Development

```bash
git clone https://github.com/pstkweb/oxidufs.git
cd oxidufs

cargo build              # debug build
cargo test               # unit + integration tests (60 tests)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Run the binary against a fixture instead of a real mergerfs mount:

```bash
OXIDUFS_FIXTURE=tests/fixtures/mountinfo_single_pool cargo run
```

The fixture format is documented in [`ARCHITECTURE.md`](ARCHITECTURE.md#fixture-file-format). Three fixtures ship in `tests/fixtures/` covering single, dual, and empty pools.

---

## Contributing

Issues and PRs are welcome.

- **Commits follow [Conventional Commits](https://www.conventionalcommits.org/).** Prefixes are used by the release pipeline (git-cliff) to categorize the changelog: `feat:`, `fix:`, `docs:`, `refactor:`, `perf:`, `test:`, `ci:`, `build:`, `chore:`. Use `feat!:` or a `BREAKING CHANGE:` footer for breaking changes.
- **CI must be green** before merge: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- **No automatic refresh, no `auto` unit mode** — these are explicit project rules, see [`CLAUDE.md`](CLAUDE.md).

---

## Releasing (maintainers)

Releases are fully automated by [`.github/workflows/release.yml`](.github/workflows/release.yml).

```bash
# 1. Bump version in Cargo.toml (must match the tag)
# 2. Commit and push
git commit -am "chore: bump version to 0.2.0"
git push origin main

# 3. Tag and push the tag
git tag v0.2.0
git push origin v0.2.0
```

The workflow:
1. Verifies `Cargo.toml` version matches the tag (fail-fast otherwise)
2. Runs the full test suite
3. Builds the release binary
4. Generates a changelog with [git-cliff](https://git-cliff.org/) from conventional commits since the previous tag
5. Creates a GitHub Release with the binary, its `.sha256`, and the changelog
6. Marks it as pre-release if the tag contains a SemVer pre-release identifier (e.g. `v0.2.0-rc.1`)

---

## Documentation

- [`docs/Roadmap.md`](docs/Roadmap.md) — phased roadmap
- [`docs/CLI-UX-Reference.md`](docs/CLI-UX-Reference.md) — full CLI surface and UX spec
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — design decisions and rationale
- [`CLAUDE.md`](CLAUDE.md) — project rules and locked decisions

---

## License

[MIT](LICENSE) © Thomas Triboult.
