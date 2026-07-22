//! Flow builder (layout container without a frame chrome).

use factorio_rs::{
    factorio_api::classes::{LuaGuiElement, LuaGuiElementAddParams},
    prelude::*,
};

use super::align::FrameDirection;
use super::widget::Widget;

/// A direction-aware layout container (`GuiElementType::Flow`).
pub struct Flow {
    name: Option<String>,
    direction: Option<FrameDirection>,
    children: Vec<Widget>,
}

impl Flow {
    /// Start a new flow builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            direction: None,
            children: Vec::new(),
        }
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

    /// Append a child widget.
    #[must_use]
    pub fn child(mut self, child: impl Into<Widget>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Create this flow (and children) under `parent`.
    pub fn mount(self, parent: LuaGuiElement) {
        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::Flow,
            ..Default::default()
        };
        if let Some(name) = self.name {
            params.name = Some(name);
        }
        if let Some(direction) = self.direction {
            params.direction = Some(direction);
        }

        let flow = parent.add(params);
        for child in self.children {
            child.mount(flow);
        }
    }
}
