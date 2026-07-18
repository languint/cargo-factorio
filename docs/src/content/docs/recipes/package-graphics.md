---
title: Package graphics
description: Copy sprites into the mod output with Factorio.toml assets and reference them by path.
---

factorio-rs builds a normal Factorio mod directory. Put graphics under your
project, list them in `Factorio.toml`, and reference the packaged paths from
data-stage (or other) code.

## 1. Add files

```text
my-mod/
  assets/graphics/icon.png
  Factorio.toml
  Cargo.toml
  src/...
```

## 2. Declare assets

```toml
[mod]
title = "My Mod"
factorio_version = "2.0"
assets = [
  { from = "assets/graphics", to = "graphics" },
]
```

Or keep the same relative path:

```toml
[mod]
assets = ["graphics"]
```

Rules (collisions, remaps, thumbnail): [Factorio.toml -> Assets](../reference/factorio-toml/#assets).

## 3. Build and check `dist/`

```bash
factorio-rs build
ls dist/graphics
```

You should see `icon.png` (or your tree) next to generated `info.json` / Lua.

## 4. Reference the Factorio path

Cargo `[package].name` is the mod id. Paths look like:

```text
__my_mod__/graphics/icon.png
```

Use that string from a **data-stage** module when defining prototypes (icons,
sprites, ...). Discover data modules with `data.rs`, `#[factorio_rs::data]`, or
`data_mod!` - see [Stages](../guides/stages/).

```rust
// Illustrative: pass the path wherever your prototype helpers expect a filename.
const ICON: &str = "__my_mod__/graphics/icon.png";
```

Replace `my_mod` with your Cargo package name.

## Thumbnail

Portal thumbnail is separate from `assets`:

```toml
[mod]
thumbnail = "assets/thumbnail.png"  # or rely on ./thumbnail.png
```

## See also

- [Getting started](../guides/getting-started/) - install / package
- [First hour](first-hour/) - end-to-end loop
