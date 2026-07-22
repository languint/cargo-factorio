//! Empty widget builder (spacer / placeholder).

use factorio_rs::{
    factorio_api::classes::{LuaGuiElement, LuaGuiElementAddParams},
    prelude::*,
};

/// An empty layout placeholder (`GuiElementType::EmptyWidget`).
pub struct EmptyWidget {
    name: Option<String>,
}

impl EmptyWidget {
    /// Start a new empty-widget builder.
    #[must_use]
    pub fn new() -> Self {
        Self { name: None }
    }

    /// Optional stable element name.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Create this empty widget under `parent`.
    #[must_use]
    pub fn mount(self, parent: LuaGuiElement) -> LuaGuiElement {
        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::EmptyWidget,
            ..Default::default()
        };
        if let Some(name) = self.name {
            params.name = Some(name);
        }
        parent.add(params)
    }
}
