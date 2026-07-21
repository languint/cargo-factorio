//! Text (label) builder.

use factorio_rs::{
    factorio_api::classes::{LuaGuiElement, LuaGuiElementAddParams},
    prelude::*,
};

/// A caption label.
pub struct Text {
    caption: String,
    name: Option<String>,
}

impl Text {
    /// Label with the given caption.
    #[must_use]
    pub fn new(caption: &str) -> Self {
        Self {
            caption: caption.into(),
            name: None,
        }
    }

    /// Optional element name.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Create this label under `parent`.
    #[must_use]
    pub fn mount(self, parent: LuaGuiElement) -> LuaGuiElement {
        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::Label,
            caption: Some(self.caption),
            ..Default::default()
        };
        if let Some(name) = self.name {
            params.name = Some(name);
        }
        parent.add(params)
    }
}
