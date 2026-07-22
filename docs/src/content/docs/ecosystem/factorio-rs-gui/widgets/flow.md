---
title: Flow
description: Flow builder - direction-aware layout container.
---

A layout container without frame chrome (`GuiElementType::Flow`).

```rust
use factorio_rs_gui::shared::flow::Flow;
use factorio_rs_gui::shared::text::Text;

Flow::new()
    .direction(GuiDirection::Vertical)
    .child(Text::new("A"))
    .child(Text::new("B"))
```

## Builder API

| Method | Effect |
| --- | --- |
| `new()` | Empty flow |
| `name(&str)` | Optional element name |
| `direction(GuiDirection)` | Horizontal / vertical layout |
| `child(impl Into<Widget>)` | Append a child |

See [Widgets](../) and [Frame](../frame/).
