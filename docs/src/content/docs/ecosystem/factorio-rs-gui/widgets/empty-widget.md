---
title: EmptyWidget
description: Empty widget builder - spacer / placeholder.
---

An empty layout placeholder (`GuiElementType::EmptyWidget`).

```rust
use factorio_rs_gui::shared::empty_widget::EmptyWidget;

EmptyWidget::new()
```

## Builder API

| Method | Effect |
| --- | --- |
| `new()` | Empty placeholder |
| `name(&str)` | Optional element name |

Useful for spacing or as a style target. See [Widgets](/ecosystem/factorio-rs-gui/widgets/).
