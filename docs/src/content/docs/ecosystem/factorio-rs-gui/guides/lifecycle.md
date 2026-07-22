---
title: Lifecycle
description: mount, install, restore, rebuild, and unmount for factorio-rs-gui.
---

The runtime owns Factorio lifecycle glue. Import from
`factorio_rs_gui::shared::runtime`.

## Mount

```rust
factorio_rs_gui::shared::runtime::mount(
    player.gui().screen(),
    "my_mod_window",
    lua_fn0(app),
);
```

- Parent is usually `player.gui().screen()`.
- `root_name` must be **mod-unique** among siblings on that parent.
- Applies `root_name` to the root [`Frame`](../../widgets/frame/) when unset.
- Stores the `app` closure, builds the tree, and wires GUI `script.on_event`
  handlers ([`ensure_events`](#events)).

## Install (hot reload + events)

After `game.reload_mods()` / hot reload, module locals wipe. Re-bind on tick
(or another safe event):

```rust
factorio_rs_gui::shared::runtime::install("my_mod_window", lua_fn0(app));
```

`install` calls [`ensure_events`](#events) then [`restore`](#advanced-restore--dispatch). Prefer
`install` over calling `restore` alone.

## Events

`mount` / `install` register these Factorio events on **your** mod's `script`
(once per Lua session):

| Event | Dispatcher | Used by |
| --- | --- | --- |
| `on_gui_click` | `dispatch_click` | [`Button`](../../widgets/button/), [`SpriteButton`](../../widgets/sprite-button/) |
| `on_gui_checked_state_changed` | `dispatch_checked` | [`Checkbox`](../../widgets/checkbox/) |
| `on_gui_text_changed` | `dispatch_text` | [`TextField`](../../widgets/text-field/) |
| `on_gui_confirmed` | `dispatch_confirmed` | [`TextField`](../../widgets/text-field/) |
| `on_gui_value_changed` | `dispatch_value` | [`Slider`](../../widgets/slider/) |
| `on_gui_selection_state_changed` | `dispatch_selection` | [`DropDown`](../../widgets/drop-down/) |

You do **not** need manual `#[factorio_rs::event(OnGui…)]` stubs for those.

Factorio's `script.on_event` **replaces** the previous handler. Do not also
define competing `#[factorio_rs::event(OnGuiClick)]` (or the other GUI events
above) in the same mod.

### Extra click logic

```rust
factorio_rs_gui::shared::runtime::on_click(lua_fn(|event: OnGuiClickEvent| {
    // Runs when no named button binding matched.
    let _ = event;
}));
```

## Rebuild and unmount

| Call | When |
| --- | --- |
| `State::set` | Marks dirty and rebuilds that root (destroy + re-run `app`) |
| `rebuild` / `rebuild_root` | Manual rebuild APIs |
| `unmount(root_name)` | Tear down one window |

v1 rebuilds the **whole** tree for a root when state changes.

## Advanced: restore / dispatch

| Call | Role |
| --- | --- |
| `restore(root, app)` | Re-bind app after reload without touching events |
| `dispatch_*` | Lower-level routers (already wired by `install`) |
| `ensure_events()` | Register GUI `script.on_event` handlers only |

## Constants

`ROOT_NAME` (`"frg_root"`) is a default for single-GUI experiments. Prefer an
explicit mod-prefixed string in real mods. See [Multiple windows](../multiple-windows/).
