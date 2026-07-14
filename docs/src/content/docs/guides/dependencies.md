---
title: Sharing code between mods
description: Export functions from one factorio-rs mod and call them from another with normal Rust.
---

Want mod B to call a function from mod A? Mark it with `#[factorio_rs::export]`,
build A, then run `factorio-rs add` from B. From there you call it like any Rust
crate: `provider::greet("hi")`.

```bash
# In the library mod
factorio-rs build

# In the dependent mod
factorio-rs add ../provider
factorio-rs build
```

Working tree: [provider / consumer example](../examples/dependencies/).

## Export a function (library mod)

Only items marked with `#[factorio_rs::export]` are visible to other mods. Plain
`pub` stays private to your mod.

```rust
#[factorio_rs::control]
mod control {
    #[factorio_rs::export]
    pub fn greet(name: &str) {
        println!("hello, {name}");
    }
}
```

You can also put `#[factorio_rs::export]` on a whole `mod` to export every `pub fn`
inside it.

After `factorio-rs build`, factorio-rs:

- Registers control exports with Factorio (`remote.add_interface`)
- Keeps shared exports loadable via Lua `require`
- Writes a small catalog at `.factorio-rs/exports.json` (gitignored)

You do **not** maintain a separate stub crate by hand.

### Control vs shared

| Where you export | How other mods call it |
| --- | --- |
| Control stage (`#[factorio_rs::control]`) | Live call into your mod: `remote.call` |
| Shared stage (`shared/...`) | Load your Lua module: `require` |

Same attribute either way - the stage picks the mechanism.

Optional: rename the Factorio remote interface:

```rust
#[factorio_rs::export(interface = "math")]
pub fn add(a: i32, b: i32) -> i32 { a + b }
```

`#[factorio_rs::export(interface)]` means “remote, use my mod name” (same default
as plain `#[export]` on control).

## Call it from another mod

From the dependent project:

```bash
factorio-rs add ../provider
```

That wires Cargo and Factorio.toml for you and generates empty type stubs under
`target/factorio-rs/bindings/provider/`. Then:

```rust
#[factorio_rs::control]
mod control {
    use provider::shared::api;

    #[factorio_rs::event(OnSingleplayerInit)]
    pub fn on_init() {
        // Control export → remote.call("provider", "greet", ...)
        provider::greet("world");

        // Shared export → require("__provider__/lua/shared/api")
        api::greet("world");
    }
}
```

Tips:

- Root names like `provider::greet` are control/remote exports.
- Paths like `provider::shared::...` are shared/`require` exports.
- Stubs are empty - your dependent mod never compiles the library’s real
  control/shared code. At runtime Factorio still does one `remote.call` or
  `require`.
- Building the dependent mod refreshes stubs if the library’s
  `.factorio-rs/exports.json` changed.

## Other Factorio deps (optional, not Rust)

If you need a non-Rust Factorio dep (DLC, optional mods, conflicts), list it in
`Factorio.toml`:

```toml
[mod]
factorio_version = "2.0"
dependencies = [
  "? space-age",
  "! some-conflict",
]
```

`factorio-rs add` also appends entries like `provider >= 0.1.3` from the library’s
export catalog. A `base >= ...` line is added automatically when missing.
`Factorio.toml` wins if the same mod is listed twice.

## Lua-only mods (flib, etc.)

Third-party mods that are not factorio-rs projects have no
`.factorio-rs/exports.json`. Hand-write a small stub crate with empty function
bodies and `[package.metadata.factorio]` (same fields the generated stubs use),
then depend on it like any Cargo path/crates.io crate.

## Troubleshooting

| Problem | Fix |
| --- | --- |
| `exports manifest missing` | Run `factorio-rs build` in the library first |
| Cargo error about two packages named `provider` | The real library and the generated stub must not both be in the same Cargo workspace. Keep library and dependent as separate projects (the examples in this repo do that) |
| Call doesn’t typecheck after changing exports | Rebuild the library, then rebuild the dependent (or re-run `factorio-rs add`) |
