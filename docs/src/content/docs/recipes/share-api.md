---
title: Share an API between mods
description: Export a function from one mod and call it from another with factorio-rs add.
---

This is a short recipe. Full reference: [Sharing code between mods](../guides/dependencies/).

## Provider: export

In the provider mod (control or shared stage):

```rust
#[factorio_rs::export]
pub fn greet(name: &str) -> String {
    format!("hello, {name}")
}
```

Build the provider. Exports are written to `.factorio-rs/exports.json` and
published for consumers (not the deprecated `emit_api` / `api_dir` keys).

## Consumer: depend and call

```bash
cd consumer-mod
factorio-rs add ../provider   # or a crates.io / git dep with Factorio metadata
```

Then:

```rust
use provider::greet;

#[factorio_rs::event(OnSingleplayerInit)]
pub fn on_singleplayer_init() {
    println!("{}", greet("world"));
}
```

Control-stage exports go through Factorio `remote`; shared-stage exports become
`require`s. The CLI generates a binding crate under `.factorio-rs/bindings/` for
typed `use` paths.

## Checklist

1. Provider has `[package.metadata.factorio]` (see the dependencies page).
2. Consumer lists the Factorio mod dependency (often via `factorio-rs add`).
3. Both mods are present in the Factorio mods folder when you run the game.

Working tree: [provider / consumer example](../examples/dependencies/).
