//! Line builder.

use factorio_rs::{
    factorio_api::classes::{LuaGuiElement, LuaGuiElementAddParams},
    prelude::*,
};

use super::align::FrameDirection;

/// A horizontal or vertical rule (`GuiElementType::Line`).
pub struct Line {
    name: Option<String>,
    direction: Option<FrameDirection>,
}

impl Line {
    /// Start a new line builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            direction: None,
        }
    }

    /// Optional stable element name.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Line orientation.
    #[must_use]
    pub fn direction(mut self, direction: FrameDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Create this line under `parent`.
    #[must_use]
    pub fn mount(self, parent: LuaGuiElement) -> LuaGuiElement {
        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::Line,
            ..Default::default()
        };
        if let Some(name) = self.name {
            params.name = Some(name);
        }
        if let Some(direction) = self.direction {
            params.direction = Some(direction);
        }
        parent.add(params)
    }
}
