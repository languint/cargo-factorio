---
title: Widgets
description: Builder widgets in factorio-rs-gui - containers, leaves, and the Widget enum.
---

Builders produce a concrete [`Widget`](/ecosystem/factorio-rs-gui/widgets/widget/) tree. Pass builders to
`Frame::child` / `Flow::child` via `impl Into<Widget>` (`From` impls on each
builder).

| Widget | Factorio element | Page |
| --- | --- | --- |
| [`Frame`](/ecosystem/factorio-rs-gui/widgets/frame/) | `frame` | Titled container, layout, caption |
| [`Flow`](/ecosystem/factorio-rs-gui/widgets/flow/) | `flow` | Layout container |
| [`ScrollPane`](/ecosystem/factorio-rs-gui/widgets/scroll-pane/) | `scroll-pane` | Scrollable container |
| [`Text`](/ecosystem/factorio-rs-gui/widgets/text/) | `label` | Caption label |
| [`Button`](/ecosystem/factorio-rs-gui/widgets/button/) | `button` | Caption + `on_click` |
| [`SpriteButton`](/ecosystem/factorio-rs-gui/widgets/sprite-button/) | `sprite-button` | Sprite + `on_click` |
| [`Checkbox`](/ecosystem/factorio-rs-gui/widgets/checkbox/) | `checkbox` | Caption + `on_checked` |
| [`TextField`](/ecosystem/factorio-rs-gui/widgets/text-field/) | `textfield` | Text input |
| [`Slider`](/ecosystem/factorio-rs-gui/widgets/slider/) | `slider` | Numeric slider |
| [`ProgressBar`](/ecosystem/factorio-rs-gui/widgets/progress-bar/) | `progressbar` | Progress display |
| [`DropDown`](/ecosystem/factorio-rs-gui/widgets/drop-down/) | `drop-down` | Item list |
| [`Sprite`](/ecosystem/factorio-rs-gui/widgets/sprite/) | `sprite` | Static sprite |
| [`Line`](/ecosystem/factorio-rs-gui/widgets/line/) | `line` | Horizontal / vertical rule |
| [`EmptyWidget`](/ecosystem/factorio-rs-gui/widgets/empty-widget/) | `empty-widget` | Spacer / placeholder |

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

Interactive widgets register handlers through [`mount` / `install`](/ecosystem/factorio-rs-gui/guides/lifecycle/)
— no manual `OnGuiClick` (or other GUI event) stubs.
