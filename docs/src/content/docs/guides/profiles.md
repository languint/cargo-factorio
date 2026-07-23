---
title: Profiles
description: Configure transpile profiles for debug comments, pruning, and IR optimization.
---

Profiles in `Factorio.toml` control **transpile** behaviour. They are not Cargo
`[profile.dev]` / `--release` profiles.

## Keys

Under `[profiles.<name>]`:

| Key | Meaning |
| --- | --- |
| `debug_level` | Lua comment verbosity (`0` = headers only; `1+` adds more inline comments) |
| `prune_dead_code` | Remove unreachable IR before codegen |
| `optimize_ir` | Rewrite IR for cheaper / more readable Lua (IIFE hoist, closure inline, ...) |

## Defaults

| Profile name | Default `debug_level` | Default `prune_dead_code` | Default `optimize_ir` |
| --- | --- | --- | --- |
| `debug` | `1` | `false` | `false` |
| anything else (including `release`) | unset (no debug-comment mode) unless TOML sets it | `true` | `true` |

Init templates and examples usually set release to `debug_level = 0` explicitly.

## What `optimize_ir` does

After optional pruning, release builds run IR passes that:

- Expand statement-context **and mid-expression** `if` / empty IIFEs into real
  `if`/`else` (temps like `__hN` for call args and binops; never Lua `and`/`or`)
- Simplify locals: Option/Result `unwrap_or`, bool if->expr, `not not` / comparison
  negation folds, `__...` copy-prop (idents/literals freely; field loads and impure
  values only for a single straight-line use so hoisted `self.tag` stays bound),
  nil-init collapse, identity pattern binds, `flag == true` -> truthiness
- Inline trivial single-use closures (e.g. `|n| n + 1` in `.map`), then simplify
  again so inlined shapes get the same folds
- Flatten nested string concatenations from `format!` / asserts (drop empty `""`
  parts)

Frontend also fuses `option.ok_or(e)?` so the Ok path skips a Result table.

Codegen emits `elseif` for nested else-if chains, shares `{ __index = Type }`
metatable locals for types **with methods**, and skips `setmetatable` on
method-less enums.

These reduce Lua 5.2 `CLOSURE` / `CALL` / `NEWTABLE` / `MOVE` traffic versus
naive IIFE emission. When investigating a hot helper, `luac -l` (Lua 5.2) on the
emitted chunk is a useful before/after check; Factorio’s opcode list matches
stock 5.2.

## CLI defaults

| Command | Default `--profile` |
| --- | --- |
| `build`, `install` | `debug` |
| `package` | `release` |
| `check` | _(no profile; lints come from `[lints]`)_ |

Override emit profiles with `--profile <name>` and/or `--debug-level N` on
`build` / `package` / `install`.

## What pruning keeps

Reachability starts from:

- `#[factorio_rs::event]` handlers
- `#[factorio_rs::export]` functions (cross-mod API must survive even if unused in-mod)
- public structs and enums in **shared** stage modules (library `require()` surface)
- public functions and structs (and enums) in settings/data stage modules
  (load-time entry points)

Everything else is kept only if referenced from those roots. Prefer passing
**function values** (or `lua_fn(...)`) into Factorio APIs instead of stringly
callback names when prune is on - otherwise the handler can be deleted.

Config reference: [Factorio.toml](/reference/factorio-toml/#profilesname).
