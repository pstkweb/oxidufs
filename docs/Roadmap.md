# OxiDuFs

> *oxidufs — a TUI for your mergerfs universe*

OxiDuFs is a terminal UI built in Rust with Ratatui, designed to give mergerfs users a clear, interactive view of their storage pools — from raw disk metrics to file distribution and health monitoring. This roadmap follows a **read-before-write** philosophy: informational features ship first, mutating operations come last with full safeguards.

---

## Phase 1 — Core (MVP)

*The reason the tool exists. Without this, nothing else matters.*

- Pool & disk overview — sizes, usage, mount points, filesystem
- Per-disk usage vs. pool-level usage
- Disk attributes — format, labels, mount options
- mergerfs aggregate config — policy, FUSE options

---

## Phase 2 — Key Differentiator

*What makes this tool unique vs. just running `df`.*

- File distribution view — which files/folders live on which disk
- Resolve real physical path of any file in the pool
- Spot imbalances visually

---

## Phase 3 — Power Features

*High value but more complex to build safely.*

- S.M.A.R.T. health status per disk
- Usage threshold warnings
- Policy simulation — where would the next write land?

---

## Phase 4 — Advanced

*Risky operations — needs solid UX and safeguards.*

- Rebalancing — preview + execute
- Policy editing

---

*Built with Rust & Ratatui · oxi → oxidation → Rust · du → disk usage · fs → filesystem*
