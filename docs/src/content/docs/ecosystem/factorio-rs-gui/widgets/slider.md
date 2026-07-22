---
title: Slider
description: Slider builder - numeric range with on_value_changed.
---

A numeric slider (`GuiElementType::Slider`).

```rust
use factorio_rs::factorio_api::lua_fn;
use factorio_rs_gui::shared::slider::Slider;

let on_change = lua_fn(move |event: OnGuiValueChangedEvent| {
    let _ = event;
});

Slider::new()
    .minimum_value(0.0)
    .maximum_value(100.0)
    .value(25.0)
    .on_value_changed(on_change)
```

## Builder API

| Method | Effect |
| --- | --- |
| `new()` | Default slider |
| `name(&str)` | Optional stable element name |
| `minimum_value` / `maximum_value` / `value` | Range and position |
| `value_step` / `discrete_values` | Stepping |
| `on_value_changed(LuaFunction)` | `on_gui_value_changed` |

See [Lifecycle](../../guides/lifecycle/).
