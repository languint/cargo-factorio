---
title: Stages
description: Map Rust modules to Factorio load stages (control, settings, data, shared).
---

Factorio loads mods in stages (`settings`, `data`, `control`, ...). factorio-rs
assigns each discovered Rust module a stage and emits the matching entry file
when needed.

## How modules are discovered

1. **Path under `src/`** (except `lib.rs`): stage from the file/directory name
   prefix. Longer names win (`settings_updates` before `settings`).
2. **Unmatched path names** (e.g. `adjacent_blacklist.rs`) default to
   **Shared** - they do not need a `shared/` folder.
3. **`lib.rs`**: not path-mapped. Use `*_mod! { ... }`, a crate attribute like
   `#![factorio_rs::control]`, or `#[factorio_rs::control] mod ... { ... }`.

## Stage table

| Stage              | Entry file                 | Declarations                                                     |
| ------------------ | -------------------------- | ---------------------------------------------------------------- |
| Settings           | `settings.lua`             | `settings.rs`, `#[factorio_rs::settings]`, `settings_mod!`       |
| SettingsUpdates    | `settings-updates.lua`     | `settings_updates` / attr / `settings_updates_mod!`              |
| SettingsFinalFixes | `settings-final-fixes.lua` | ...                                                              |
| Data               | `data.lua`                 | `data.rs`, `#[factorio_rs::data]`, `data_mod!`                   |
| DataUpdates        | `data-updates.lua`         | ...                                                              |
| DataFinalFixes     | `data-final-fixes.lua`     | ...                                                              |
| Control            | `control.lua`              | `control.rs`, `#[factorio_rs::control]`, `control_mod!`          |
| Shared             | _(none)_                   | `shared.rs`, `#[factorio_rs::shared]`, `shared_mod!`, or default |

Shared modules are required by other stages; they have no Factorio entry file.

## Side-effect entry points

For settings and data stages, every **`pub fn`** in the module is called from
the stage entry Lua at load time. That is why `mod_settings!` generates
`pub fn register()` - `settings.lua` ends up calling it.

## Events only in control

`#[factorio_rs::event]` handlers are only allowed in **Control** modules. Putting
them elsewhere is a frontend error.
