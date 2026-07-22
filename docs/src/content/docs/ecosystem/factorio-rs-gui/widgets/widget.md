---
title: Widget
description: The Widget enum - mountable container and leaf nodes.
---

`Widget` is the concrete tree type the runtime mounts. Variants cover every
builder (`Frame`, `Flow`, `Button`, `Checkbox`, …). See the
[widgets overview](../) for the full list.

## Conversions

Each builder implements `Into<Widget>` / `From`, so `Frame::child` /
`Flow::child` accept builders directly:

```rust
.child(Text::new("hi"))
.child(Button::new("Go"))
.child(Flow::new().direction(GuiDirection::Horizontal).child(Text::new("nested")))
```

Return `impl Into<Widget>` from your `app` function (usually a root `Frame`).

## Mount

```rust
widget.mount(parent); // creates Factorio elements under `parent`
```

[`runtime::mount`](../../guides/lifecycle/) wraps this for screen roots and
applies the mount `root_name` when the root frame has no explicit `.name(...)`.
