---
title: DropDown
description: Drop-down builder - items list with on_selection_changed.
---

A drop-down list (`GuiElementType::DropDown`).

```rust
use factorio_rs::factorio_api::lua_fn;
use factorio_rs_gui::shared::drop_down::DropDown;

let on_pick = lua_fn(move |event: OnGuiSelectionStateChangedEvent| {
    let _ = event;
});

DropDown::new(vec!["A".into(), "B".into(), "C".into()])
    .selected_index(1)
    .on_selection_changed(on_pick)
```

`selected_index` is **1-based** (Factorio convention).

## Builder API

| Method | Effect |
| --- | --- |
| `new(Vec<String>)` | Items |
| `name(&str)` | Optional stable element name |
| `selected_index(u32)` | Initial selection (1-based) |
| `on_selection_changed(LuaFunction)` | `on_gui_selection_state_changed` |

See [Lifecycle](/ecosystem/factorio-rs-gui/guides/lifecycle/).
