---
title: TextField
description: Text field builder - text input with change and confirm handlers.
---

A single-line text field (`GuiElementType::Textfield`).

```rust
use factorio_rs::factorio_api::lua_fn;
use factorio_rs_gui::shared::text_field::TextField;

let on_change = lua_fn(move |event: OnGuiTextChangedEvent| {
    let _ = event.text;
});

TextField::new().text("hello").on_text_changed(on_change)
```

## Builder API

| Method | Effect |
| --- | --- |
| `new()` | Empty field |
| `text(&str)` | Initial contents |
| `name(&str)` | Optional stable element name |
| `numeric(bool)` / `allow_decimal` / `allow_negative` | Numeric mode |
| `is_password(bool)` | Mask input |
| `lose_focus_on_confirm(bool)` | Blur on Enter |
| `on_text_changed(LuaFunction)` | `on_gui_text_changed` |
| `on_confirmed(LuaFunction)` | `on_gui_confirmed` |

See [Lifecycle](../../guides/lifecycle/).
