---
title: Line
description: Line builder - horizontal or vertical rule.
---

A layout rule (`GuiElementType::Line`).

```rust
use factorio_rs_gui::shared::line::Line;

Line::new().direction(GuiDirection::Horizontal)
```

## Builder API

| Method | Effect |
| --- | --- |
| `new()` | Default line |
| `name(&str)` | Optional element name |
| `direction(GuiDirection)` | Orientation |

See [Widgets](/ecosystem/factorio-rs-gui/widgets/).
