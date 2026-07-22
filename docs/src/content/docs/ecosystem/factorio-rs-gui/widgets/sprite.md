---
title: Sprite
description: Sprite builder - static sprite path.
---

A static sprite (`GuiElementType::Sprite`).

```rust
use factorio_rs_gui::shared::sprite::Sprite;

Sprite::new("item/iron-plate").resize_to_sprite(true)
```

## Builder API

| Method | Effect |
| --- | --- |
| `new(&'static str)` | Sprite path |
| `name(&str)` | Optional element name |
| `resize_to_sprite(bool)` | Size to sprite |

See [SpriteButton](../sprite-button/) for a clickable variant.
