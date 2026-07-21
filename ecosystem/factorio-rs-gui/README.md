# factorio-rs-gui

Reactive, builder-style GUI helpers for [factorio-rs](https://crates.io/crates/factorio-rs) mods.

Docs: <https://languint.github.io/factorio-rs/> (recipe: Reactive GUI).

## Example

```rust
use factorio_rs::factorio_api::{lua_fn, lua_fn0};
use factorio_rs::prelude::*;
use factorio_rs_gui::shared::button::Button;
use factorio_rs_gui::shared::frame::Frame;
use factorio_rs_gui::shared::text::Text;
use factorio_rs_gui::shared::widget::Widget;

fn app() -> Widget {
    let count = factorio_rs_gui::state!(0);
    let label = format!("Count: {}", count.get());
    let increment = lua_fn(move |event: OnGuiClickEvent| {
        let _ = event;
        count.set(count.get() + 1);
    });

    Frame::new()
        .caption("Counter")
        .align_horizontal(LuaStyleHorizontalAlign::Center)
        .align_vertical(LuaStyleVerticalAlign::Center)
        .direction(GuiDirection::Vertical)
        .child(Text::new(&label).as_widget())
        .child(Button::new("Increment").on_click(increment).as_widget())
        .as_widget()
}

// factorio_rs_gui::shared::runtime::mount(player.gui().screen(), lua_fn0(app));

// Also forward clicks from *your* OnGuiClick:
// factorio_rs_gui::shared::runtime::dispatch_click(event);
```

v1 rebuilds the whole tree when state changes (clear + re-run `app`).
Handlers live in the consuming mod's `storage`, so you must call
`dispatch_click` from your own `OnGuiClick` handler.

```bash
cd ecosystem/factorio-rs-gui && factorio-rs build
cd examples/gui_counter && factorio-rs build
```
