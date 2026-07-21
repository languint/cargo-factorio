---
title: Reactive GUI
description: Build a reactive Factorio GUI with factorio-rs-gui builders, state!, and mount.
---

Use the ecosystem crate [`factorio-rs-gui`](https://github.com/languint/factorio-rs/tree/main/ecosystem/factorio-rs-gui)
for a builder-style UI that rebuilds when state changes.

## Shape

```rust
use factorio_rs::{
    factorio_api::{IndexOrName, lua_fn, lua_fn0},
    prelude::*,
};
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

#[factorio_rs::event(OnPlayerCreated)]
pub fn on_player_created(event: OnPlayerCreatedEvent) {
    if let Some(player) = game.get_player(IndexOrName::Index(event.player_index)) {
        factorio_rs_gui::shared::runtime::mount(player.gui().screen(), lua_fn0(app));
    }
}

#[factorio_rs::event(OnGuiClick)]
pub fn on_gui_click(event: OnGuiClickEvent) {
    factorio_rs_gui::shared::runtime::dispatch_click(event);
}
```

## How it works

1. `state!(init)` allocates a hook slot that survives rebuilds.
2. `mount(parent, app)` stores the app function and builds the tree.
3. Button `on_click` registers handlers in **your** mod's `storage`; your
   `OnGuiClick` must call `dispatch_click`.
4. `State::set` marks dirty and **rebuilds** (clear + re-run `app`).

Adaptations from a fully reactive DSL: use `format!` (not `"Count: {count}"`
literals), `lua_fn` / `lua_fn0` for callbacks, `.as_widget()` to wrap children,
and a concrete `Widget` enum (not `impl Trait` from another mod). Prefer
`factorio_rs_gui::shared::...` paths so Factorio `require`s resolve.

## Try it

```bash
cd ecosystem/factorio-rs-gui && factorio-rs build
cd examples/gui_counter
factorio-rs build && factorio-rs install --open
```

Working tree: [`examples/gui_counter`](../examples/gui-counter/).

## See also

- [GUI basics](gui-basics/) - imperative `LuaGuiElementAddParams`
- [Sharing code between mods](../guides/dependencies/) - library deps
- [Persist with storage](persist-storage/) - hook values use `storage`
