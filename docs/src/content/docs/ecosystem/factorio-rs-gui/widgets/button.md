---
title: Button
description: Button builder - caption, optional name, and on_click handlers.
---

A clickable button (`GuiElementType::Button`).

```rust
use factorio_rs::factorio_api::lua_fn;
use factorio_rs_gui::shared::button::Button;

let on_ok = lua_fn(move |event: OnGuiClickEvent| {
    let _ = event;
    // ...
});

Button::new("OK").on_click(on_ok)
```

## Builder API

| Method | Effect |
| --- | --- |
| `new(&str)` | Button with caption |
| `name(&str)` | Optional stable element name |
| `on_click(LuaFunction)` | Click handler (`lua_fn` / `lua_fn0` / function item) |

If `on_click` is set and you omit `.name(...)`, the runtime assigns a unique
name (`frg_btn...`) so the click can be routed.

## Events

`mount` / `install` register `dispatch_click` on your mod's `script`. You do not
need a manual `OnGuiClick` stub. For extra click logic (when no named button
matched), use `runtime::on_click`.

See [Lifecycle](/ecosystem/factorio-rs-gui/guides/lifecycle/).
