//! Drop-down builder.

use factorio_rs::{
    factorio_api::{
        LuaFunction,
        classes::{LuaGuiElement, LuaGuiElementAddParams},
    },
    prelude::*,
};

/// A drop-down list (`GuiElementType::DropDown`).
pub struct DropDown {
    name: Option<String>,
    items: Vec<String>,
    selected_index: Option<u32>,
    on_selection_changed: Option<LuaFunction>,
}

impl DropDown {
    /// Drop-down with the given items.
    #[must_use]
    pub fn new(items: Vec<String>) -> Self {
        Self {
            name: None,
            items,
            selected_index: None,
            on_selection_changed: None,
        }
    }

    /// Optional stable element name (auto-assigned when a handler is set).
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Initially selected item (1-based Factorio index).
    #[must_use]
    pub fn selected_index(mut self, index: u32) -> Self {
        self.selected_index = Some(index);
        self
    }

    /// Register a selection-changed handler.
    #[must_use]
    pub fn on_selection_changed(mut self, handler: LuaFunction) -> Self {
        self.on_selection_changed = Some(handler);
        self
    }

    /// Create this drop-down under `parent` and register its handler.
    #[must_use]
    pub fn mount(self, parent: LuaGuiElement) -> LuaGuiElement {
        let name = match self.name {
            Some(name) => name,
            None => {
                if self.on_selection_changed.is_some() {
                    crate::shared::runtime::next_element_name("frg_dd")
                } else {
                    String::new()
                }
            }
        };

        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::DropDown,
            items: Some(self.items),
            ..Default::default()
        };
        if !name.is_empty() {
            params.name = Some(name.clone());
        }
        if let Some(index) = self.selected_index {
            params.selected_index = Some(index);
        }

        let dropdown = parent.add(params);
        if let Some(handler) = self.on_selection_changed {
            crate::shared::runtime::register_selection(name, handler);
        }
        dropdown
    }
}
