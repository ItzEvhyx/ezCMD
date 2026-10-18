# ezCMD 2.0.0 release checks

Recorded on 2026-10-18 for the Version 2 release candidate.

## Build and quality gates

| Check | Result |
|:---|:---|
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --all-targets -- -D warnings` | Pass |
| `cargo test --all-targets` | Pass |
| `cargo build --release` | Pass |
| `python3 tools/md_to_toml.py --check` | Pass; 62 imported tools, 286 imported sections, 1,639 imported command entries |
| Draft data files | Pass; none remain |
| Runtime dependencies | Pass; `strsim` and `terminal_size` only |

## Library inventory

- Runtime tools: 63
- Runtime sections: 308
- Runtime command entries: 1,737

The specification's final-library statement says 307 sections and 1,639 command entries. The generated v2 import itself has the documented 286 sections and 1,639 entries. Runtime content also preserves 12 legacy Git sections and 10 legacy Shell sections, which produces 308 sections and 1,737 entries. This is intentional: the migration requirement says to retain the v1 Git and Shell content exactly.

## Performance and size

| Check | Target | Result |
|:---|:---|:---|
| Release binary | < 3 MB | 865,248 bytes (pass) |
| Startup: `ezcmd --help` | < 20 ms | 1.38 ms mean across 50 local wall-clock runs (pass) |
| Startup: `ezcmd initializing repo` | < 20 ms | 19.75 ms mean across 50 local wall-clock runs (pass) |
| Peak memory | < 10 MB | 11.88 MiB mean peak resident memory across 50 isolated searches (miss) |

`hyperfine` is unavailable in this environment, as it was at V0, so startup timings use `date +%s%N` around the release binary and are not directly comparable with a hyperfine baseline. The V0 baseline has no startup timing, so the plan's “more than 5 ms from baseline” condition for indexing cannot be evaluated. The matcher now avoids edit-distance calculations that cannot meet the permitted distance threshold, bringing the tested query below the startup budget. Peak memory was sampled with Linux child-process resource accounting because `/usr/bin/time` is unavailable; it remains above the documented budget.

## Help shape and platform coverage

- `ezcmd --help` contains zero `docker run`, `git commit`, or `npm install` command lines.
- `ezcmd --help -docker` contains zero `docker run` command lines.
- The checked-in CI workflow runs formatting, clippy, tests, and a release build on Linux, macOS, and Windows. Its 3 MB size gate is present for Unix and Windows.
- The release workflow is configured for five artifacts: Linux x86_64 and aarch64, macOS x86_64 and aarch64, and Windows x86_64.

The CI and release workflows were inspected but not triggered from this local verification; creating a test tag would publish an external release and requires explicit release authorization.
