---
title: Widgets
description: Builder widgets in factorio-rs-gui - containers, leaves, and the Widget enum.
---

Builders produce a concrete [`Widget`](widget/) tree. Pass builders to
`Frame::child` / `Flow::child` via `impl Into<Widget>` (`From` impls on each
builder).

| Widget | Factorio element | Page |
| --- | --- | --- |
| [`Frame`](frame/) | `frame` | Titled container, layout, caption |
| [`Flow`](flow/) | `flow` | Layout container |
| [`ScrollPane`](scroll-pane/) | `scroll-pane` | Scrollable container |
| [`Text`](text/) | `label` | Caption label |
| [`Button`](button/) | `button` | Caption + `on_click` |
| [`SpriteButton`](sprite-button/) | `sprite-button` | Sprite + `on_click` |
| [`Checkbox`](checkbox/) | `checkbox` | Caption + `on_checked` |
| [`TextField`](text-field/) | `textfield` | Text input |
| [`Slider`](slider/) | `slider` | Numeric slider |
| [`ProgressBar`](progress-bar/) | `progressbar` | Progress display |
| [`DropDown`](drop-down/) | `drop-down` | Item list |
| [`Sprite`](sprite/) | `sprite` | Static sprite |
| [`Line`](line/) | `line` | Horizontal / vertical rule |
| [`EmptyWidget`](empty-widget/) | `empty-widget` | Spacer / placeholder |

```rust
use factorio_rs_gui::shared::button::Button;
use factorio_rs_gui::shared::flow::Flow;
use factorio_rs_gui::shared::frame::Frame;
use factorio_rs_gui::shared::text::Text;
use factorio_rs_gui::shared::widget::Widget;

fn app() -> impl Into<Widget> {
    Frame::new()
        .caption("Hello")
        .direction(GuiDirection::Vertical)
        .child(
            Flow::new()
                .direction(GuiDirection::Vertical)
                .child(Text::new("Label"))
                .child(Button::new("OK")),
        )
}
```

Prefer `factorio_rs_gui::shared::...` paths (not a flattened prelude import alone)
so Factorio `require`s resolve correctly after transpile.

Interactive widgets register handlers through [`mount` / `install`](../guides/lifecycle/)
— no manual `OnGuiClick` (or other GUI event) stubs.
