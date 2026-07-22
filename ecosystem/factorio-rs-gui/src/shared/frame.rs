//! Frame builder.

use factorio_rs::{
    factorio_api::classes::{LuaGuiElement, LuaGuiElementAddParams},
    prelude::*,
};

// `super::` keeps these Rust-only: `align` is type aliases with no Lua module,
// and `crate::` would emit a missing `require(".../shared/align")`.
use super::align::{FrameDirection, HorizontalAlignment, VerticalAlignment};
// `super::` also avoids a load cycle with `widget` -> `frame`.
use super::widget::Widget;

/// A titled container that lays out children.
pub struct Frame {
    caption: Option<String>,
    name: Option<String>,
    direction: Option<FrameDirection>,
    align_horizontal: Option<HorizontalAlignment>,
    align_vertical: Option<VerticalAlignment>,
    /// Distinct from the [`Self::auto_center`] builder method so Lua field
    /// lookup does not shadow the metatable method of the same name.
    center_on_mount: bool,
    children: Vec<Widget>,
}

impl Frame {
    /// Start a new frame builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            caption: None,
            name: None,
            direction: None,
            align_horizontal: None,
            align_vertical: None,
            center_on_mount: false,
            children: Vec::new(),
        }
    }

    /// Frame title caption.
    #[must_use]
    pub fn caption(mut self, caption: &str) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Optional stable element name.
    ///
    /// Root frames mounted via [`crate::shared::runtime::mount`] receive the
    /// mount `root_name` automatically when unset, you usually omit this.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set `name` only when the builder has none yet (used by the mount runtime).
    #[must_use]
    pub fn ensure_name(mut self, name: &str) -> Self {
        if self.name.is_none() {
            self.name = Some(name.into());
        }
        self
    }

    /// Layout direction (`horizontal` / `vertical`).
    #[must_use]
    pub fn direction(mut self, direction: FrameDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Horizontal alignment via [`LuaStyle`].
    #[must_use]
    pub fn align_horizontal(mut self, align: HorizontalAlignment) -> Self {
        self.align_horizontal = Some(align);
        self
    }

    /// Vertical alignment via [`LuaStyle`].
    #[must_use]
    pub fn align_vertical(mut self, align: VerticalAlignment) -> Self {
        self.align_vertical = Some(align);
        self
    }

    /// Center this frame in [`LuaGui::screen`] after mount (`force_auto_center`).
    ///
    /// Only applies when the frame is a direct child of `player.gui.screen`.
    /// Rebuild restores a saved drag location afterward, so this does not fight
    /// position persistence across state updates.
    ///
    /// Named `centered` (not `auto_center`) so it does not collide with
    /// [`LuaGuiElement::auto_center`](factorio_rs::factorio_api::classes::LuaGuiElement::auto_center)
    /// in codegen call lists.
    #[must_use]
    pub fn centered(mut self) -> Self {
        self.center_on_mount = true;
        self
    }

    /// Append a child widget.
    #[must_use]
    pub fn child(mut self, child: impl Into<Widget>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Create this frame (and children) under `parent`.
    pub fn mount(self, parent: LuaGuiElement) {
        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::Frame,
            ..Default::default()
        };
        if let Some(caption) = self.caption {
            params.caption = Some(caption);
        }
        if let Some(name) = self.name {
            params.name = Some(name);
        }
        if let Some(direction) = self.direction {
            params.direction = Some(direction);
        }

        let frame = parent.add(params);
        let style = frame.style();
        if let Some(align) = self.align_horizontal {
            style.set_horizontal_align(align);
        }
        if let Some(align) = self.align_vertical {
            style.set_vertical_align(align);
        }

        for child in self.children {
            child.mount(frame);
        }

        if self.center_on_mount {
            // `auto_center` keeps the frame centered across window resizes;
            // `force_auto_center` applies immediately (size is known after children).
            frame.set_auto_center(true);
            frame.force_auto_center();
        }
    }
}
