---
title: SpriteButton
description: Sprite button builder - sprite path with on_click.
---

A clickable sprite button (`GuiElementType::SpriteButton`).

```rust
use factorio_rs::factorio_api::lua_fn;
use factorio_rs_gui::shared::sprite_button::SpriteButton;

let on_ok = lua_fn(move |event: OnGuiClickEvent| {
    let _ = event;
});

SpriteButton::new("utility/close_black").on_click(on_ok)
```

Clicks reuse the same `dispatch_click` path as [`Button`](../button/).

## Builder API

| Method | Effect |
| --- | --- |
| `new(&'static str)` | Default sprite path |
| `name(&str)` | Optional stable element name |
| `clicked_sprite` / `hovered_sprite` | Alternate sprites |
| `number(f64)` | Optional badge |
| `on_click(LuaFunction)` | Click handler |

See [Lifecycle](../../guides/lifecycle/).
