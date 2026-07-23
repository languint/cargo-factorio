---
title: ProgressBar
description: Progress bar builder - value in 0..1 with optional caption.
---

A progress bar (`GuiElementType::Progressbar`).

```rust
use factorio_rs_gui::shared::progress_bar::ProgressBar;

ProgressBar::new().value(0.5).caption("50%")
```

## Builder API

| Method | Effect |
| --- | --- |
| `new()` | Empty bar |
| `name(&str)` | Optional element name |
| `value(f64)` | Progress in `[0.0, 1.0]` |
| `caption(&str)` | Overlay caption |

No interactive events. See [Widgets](/ecosystem/factorio-rs-gui/widgets/).
