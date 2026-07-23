---
title: Checkbox
description: Checkbox builder - caption, state, and on_checked handlers.
---

A checkbox (`GuiElementType::Checkbox`).

```rust
use factorio_rs::factorio_api::lua_fn;
use factorio_rs_gui::shared::checkbox::Checkbox;

let on_toggle = lua_fn(move |event: OnGuiCheckedStateChangedEvent| {
    let _ = event;
});

Checkbox::new("Enabled").state(true).on_checked(on_toggle)
```

## Builder API

| Method | Effect |
| --- | --- |
| `new(&str)` | Caption |
| `name(&str)` | Optional stable element name |
| `state(bool)` | Initial checked state |
| `on_checked(LuaFunction)` | `on_gui_checked_state_changed` handler |

`mount` / `install` wire the checked-state dispatcher. See [Lifecycle](/ecosystem/factorio-rs-gui/guides/lifecycle/).
