---
title: tracing_test
description: Control-stage example using the optional tracing feature.
---

Path: `examples/tracing_test`.

Enables `factorio-rs` feature `tracing` and logs colored messages on singleplayer
init:

```rust
factorio_rs::control_mod! {
    use factorio_rs::tracing;

    #[factorio_rs::event(OnSingleplayerInit)]
    pub fn on_singleplayer_init() {
        tracing::info!("Hello factorio-rs!");
        tracing::error!("Oopsies!");
    }
}
```

## Try it

```bash
cd examples/tracing_test
factorio-rs build
factorio-rs install --open   # optional
```

Requires a CLI build with the default `tracing` feature (included in normal
`cargo install factorio-rs-cli`).

See [Tracing](../guides/tracing/) for colors, format limits, and feature wiring.
