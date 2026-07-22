//! Scroll pane builder.

use factorio_rs::{
    factorio_api::classes::{LuaGuiElement, LuaGuiElementAddParams},
    prelude::*,
};

use super::widget::Widget;

/// A scrollable container (`GuiElementType::ScrollPane`).
pub struct ScrollPane {
    name: Option<String>,
    horizontal_scroll_policy: Option<ScrollPolicy>,
    vertical_scroll_policy: Option<ScrollPolicy>,
    children: Vec<Widget>,
}

impl ScrollPane {
    /// Start a new scroll-pane builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            horizontal_scroll_policy: None,
            vertical_scroll_policy: None,
            children: Vec::new(),
        }
    }

    /// Optional stable element name.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Horizontal scroll policy.
    #[must_use]
    pub fn horizontal_scroll_policy(mut self, policy: ScrollPolicy) -> Self {
        self.horizontal_scroll_policy = Some(policy);
        self
    }

    /// Vertical scroll policy.
    #[must_use]
    pub fn vertical_scroll_policy(mut self, policy: ScrollPolicy) -> Self {
        self.vertical_scroll_policy = Some(policy);
        self
    }

    /// Append a child widget.
    #[must_use]
    pub fn child(mut self, child: impl Into<Widget>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Create this scroll pane (and children) under `parent`.
    pub fn mount(self, parent: LuaGuiElement) {
        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::ScrollPane,
            ..Default::default()
        };
        if let Some(name) = self.name {
            params.name = Some(name);
        }
        if let Some(policy) = self.horizontal_scroll_policy {
            params.horizontal_scroll_policy = Some(policy);
        }
        if let Some(policy) = self.vertical_scroll_policy {
            params.vertical_scroll_policy = Some(policy);
        }

        let pane = parent.add(params);
        for child in self.children {
            child.mount(pane);
        }
    }
}
