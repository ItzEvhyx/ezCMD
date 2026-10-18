# ezCMD v2 baseline

Recorded before implementing Version 2 on 2026-10-18.

## Current health

- Package version: `0.2.1`
- Test suite: `cargo test --all-targets` passes (40 unit tests, 33 integration/snapshot tests, and the accuracy suite)
- Release build: passes; `target/release/ezcmd` is 410,296 bytes
- `python3` is available for the v2 converter
- `hyperfine` and `cargo-bloat` are not installed in this environment, so V0 cannot record startup timings or bloat output. V10 should run those measurements in a release environment that provides them.

## Shipped code versus IMPLEMENTATION.md

The shipped v0.2.1 implementation follows the guide's Rust, generated-TOML, offline, and OS-resolution architecture, but includes features not covered by the original guide:

- `Action` additionally supports `Install`, `Welcome`, and `Uninstall { yes }`; `--unintall` remains accepted as a legacy typo for uninstall.
- `src/welcome.rs` and `src/uninstall.rs`, plus `install.sh` and `install.ps1`, implement installation, welcome output, and removal.
- `data/shell.toml` adds a second tool beyond Git; the runtime table has the `git` and `shell` tools.
- `main.rs` presents the extra actions in usage output and currently implements v1's full command-dump help.
- `prompt.rs`, terminal coloring, snapshots, and integration tests are more complete than the module sketches in the original guide.
- The current content and tests use `TOOLS`, flat `Tool.sections`, and the v1 help parser. Version 2 must migrate these paths without removing the shipped actions.

## Reserved names before the v2 import

- Tool slugs: `git`, `shell`
- Tool aliases: none (v1 has no tool-level alias field)
- OS selectors: `windows`, `linux`, `macos`
- Long actions: `--help`, `--plain`, `--version`, `--install`, `--uninstall`, `--welcome`, `--yes`

The v2 build validation must reserve the tool slugs and OS selectors while importing new tool flags and aliases.
