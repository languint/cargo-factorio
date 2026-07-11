---
title: CLI
description: Reference for the factorio-rs command-line interface.
---

Binary name: **`factorio-rs`** (from crate **`factorio-rs-cli`**).

## Commands

### `factorio-rs init`

Create a new project in the current directory (or `--manifest-path`).

| Flag | Description |
| --- | --- |
| `--name <NAME>` | Cargo package name (default: directory name) |
| `--manifest-path <PATH>` | Project directory or `Factorio.toml` |

### `factorio-rs build`

Transpile `source` into `output_dir`.

| Flag | Description |
| --- | --- |
| `--manifest-path <PATH>` | Project directory or `Factorio.toml` |
| `--profile <NAME>` | Default: `debug` |
| `--debug-level <N>` | Override profile debug comments |
| `--package` | Also write `{name}_{version}.zip` after building |

### `factorio-rs package`

Build then create a Factorio-ready zip at the project root.

| Flag | Description |
| --- | --- |
| `--manifest-path <PATH>` | Project directory or `Factorio.toml` |
| `--profile <NAME>` | Default: `release` |
| `--debug-level <N>` | Override profile debug comments |

### `factorio-rs install`

Build and copy `output_dir` to `{mods_dir}/{name}_{version}/`.

| Flag | Description |
| --- | --- |
| `--manifest-path <PATH>` | Project directory or `Factorio.toml` |
| `--profile <NAME>` | Default: `debug` |
| `--debug-level <N>` | Override profile debug comments |
| `--open` | Launch Factorio after installing |

Mods directory: `FACTORIO_MODS_DIR` or `~/.factorio/mods`.

### `factorio-rs open`

Launch Factorio if detected (`FACTORIO_PATH`, Steam installs, PATH, or Steam
protocol). Prefers `steam-run` when available.
