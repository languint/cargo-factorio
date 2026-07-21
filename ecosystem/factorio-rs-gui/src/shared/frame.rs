//! Frame builder.

use factorio_rs::{
    factorio_api::classes::{LuaGuiElement, LuaGuiElementAddParams},
    prelude::*,
};

// `super::` keeps these Rust-only: `align` is type aliases with no Lua module,
// and `crate::` would emit a missing `require("…/shared/align")`.
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
    auto_center: bool,
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
            auto_center: false,
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
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
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
    #[must_use]
    pub fn auto_center(mut self) -> Self {
        self.auto_center = true;
        self
    }

    /// Append a child widget.
    #[must_use]
    pub fn child(mut self, child: Widget) -> Self {
        self.children.push(child);
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

        if self.auto_center {
            frame.force_auto_center();
        }
    }
}
