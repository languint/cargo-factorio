//! Checkbox builder.

use factorio_rs::{
    factorio_api::{
        LuaFunction,
        classes::{LuaGuiElement, LuaGuiElementAddParams},
    },
    prelude::*,
};

/// A checkbox with optional checked-state handler.
pub struct Checkbox {
    caption: String,
    name: Option<String>,
    state: Option<bool>,
    on_checked: Option<LuaFunction>,
}

impl Checkbox {
    /// Checkbox with the given caption.
    #[must_use]
    pub fn new(caption: &str) -> Self {
        Self {
            caption: caption.into(),
            name: None,
            state: None,
            on_checked: None,
        }
    }

    /// Optional stable element name (auto-assigned when a handler is set).
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Initial checked state.
    #[must_use]
    pub fn state(mut self, state: bool) -> Self {
        self.state = Some(state);
        self
    }

    /// Register a checked-state handler (`lua_fn(|event| { ... })`).
    #[must_use]
    pub fn on_checked(mut self, handler: LuaFunction) -> Self {
        self.on_checked = Some(handler);
        self
    }

    /// Create this checkbox under `parent` and register its handler.
    #[must_use]
    pub fn mount(self, parent: LuaGuiElement) -> LuaGuiElement {
        let name = match self.name {
            Some(name) => name,
            None => {
                if self.on_checked.is_some() {
                    crate::shared::runtime::next_element_name("frg_chk")
                } else {
                    String::new()
                }
            }
        };

        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::Checkbox,
            caption: Some(self.caption),
            ..Default::default()
        };
        if !name.is_empty() {
            params.name = Some(name.clone());
        }
        if let Some(state) = self.state {
            params.state = Some(state);
        }

        let checkbox = parent.add(params);
        if let Some(handler) = self.on_checked {
            crate::shared::runtime::register_checked(name, handler);
        }
        checkbox
    }
}
