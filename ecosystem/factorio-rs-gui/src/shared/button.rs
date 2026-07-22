//! Button builder with click handlers.

use factorio_rs::{
    factorio_api::{
        LuaFunction,
        classes::{LuaGuiElement, LuaGuiElementAddParams},
    },
    prelude::*,
};

/// A clickable button.
pub struct Button {
    caption: String,
    name: Option<String>,
    on_click: Option<LuaFunction>,
}

impl Button {
    /// Button with the given caption.
    #[must_use]
    pub fn new(caption: &str) -> Self {
        Self {
            caption: caption.into(),
            name: None,
            on_click: None,
        }
    }

    /// Optional stable element name (auto-assigned when an `on_click` is set).
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Register a click handler (`lua_fn(|event| { ... })` or a function item).
    #[must_use]
    pub fn on_click(mut self, handler: LuaFunction) -> Self {
        self.on_click = Some(handler);
        self
    }

    /// Create this button under `parent` and register its click handler.
    #[must_use]
    pub fn mount(self, parent: LuaGuiElement) -> LuaGuiElement {
        let name = match self.name {
            Some(name) => name,
            None => {
                if self.on_click.is_some() {
                    crate::shared::runtime::next_element_name("frg_btn")
                } else {
                    String::new()
                }
            }
        };

        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::Button,
            caption: Some(self.caption),
            ..Default::default()
        };
        if !name.is_empty() {
            params.name = Some(name.clone());
        }

        let button = parent.add(params);
        if let Some(handler) = self.on_click {
            crate::shared::runtime::register_click(name, handler);
        }
        button
    }
}
