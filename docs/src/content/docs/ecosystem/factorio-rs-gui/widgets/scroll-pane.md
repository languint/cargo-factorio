---
title: ScrollPane
description: Scroll pane builder - scrollable container with policy setters.
---

A scrollable container (`GuiElementType::ScrollPane`).

```rust
use factorio_rs_gui::shared::scroll_pane::ScrollPane;
use factorio_rs_gui::shared::text::Text;

ScrollPane::new()
    .vertical_scroll_policy(ScrollPolicy::Always)
    .child(Text::new("Long content…"))
```

## Builder API

| Method | Effect |
| --- | --- |
| `new()` | Empty scroll pane |
| `name(&str)` | Optional element name |
| `horizontal_scroll_policy(ScrollPolicy)` | Horizontal policy |
| `vertical_scroll_policy(ScrollPolicy)` | Vertical policy |
| `child(impl Into<Widget>)` | Append a child |

See [Widgets](../).
