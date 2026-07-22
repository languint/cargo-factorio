//! Slider builder.

use factorio_rs::{
    factorio_api::{
        LuaFunction,
        classes::{LuaGuiElement, LuaGuiElementAddParams},
    },
    prelude::*,
};

/// A numeric slider (`GuiElementType::Slider`).
pub struct Slider {
    name: Option<String>,
    minimum_value: Option<f64>,
    maximum_value: Option<f64>,
    value: Option<f64>,
    value_step: Option<f64>,
    discrete_values: Option<bool>,
    on_value_changed: Option<LuaFunction>,
}

impl Slider {
    /// Start a new slider builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            minimum_value: None,
            maximum_value: None,
            value: None,
            value_step: None,
            discrete_values: None,
            on_value_changed: None,
        }
    }

    /// Optional stable element name (auto-assigned when a handler is set).
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Minimum slider value.
    #[must_use]
    pub fn minimum_value(mut self, value: f64) -> Self {
        self.minimum_value = Some(value);
        self
    }

    /// Maximum slider value.
    #[must_use]
    pub fn maximum_value(mut self, value: f64) -> Self {
        self.maximum_value = Some(value);
        self
    }

    /// Initial slider value.
    #[must_use]
    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }

    /// Step size between values.
    #[must_use]
    pub fn value_step(mut self, step: f64) -> Self {
        self.value_step = Some(step);
        self
    }

    /// Snap to discrete steps.
    #[must_use]
    pub fn discrete_values(mut self, discrete: bool) -> Self {
        self.discrete_values = Some(discrete);
        self
    }

    /// Register a value-changed handler.
    #[must_use]
    pub fn on_value_changed(mut self, handler: LuaFunction) -> Self {
        self.on_value_changed = Some(handler);
        self
    }

    /// Create this slider under `parent` and register its handler.
    #[must_use]
    pub fn mount(self, parent: LuaGuiElement) -> LuaGuiElement {
        let name = match self.name {
            Some(name) => name,
            None => {
                if self.on_value_changed.is_some() {
                    crate::shared::runtime::next_element_name("frg_sld")
                } else {
                    String::new()
                }
            }
        };

        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::Slider,
            ..Default::default()
        };
        if !name.is_empty() {
            params.name = Some(name.clone());
        }
        if let Some(value) = self.minimum_value {
            params.minimum_value = Some(value);
        }
        if let Some(value) = self.maximum_value {
            params.maximum_value = Some(value);
        }
        if let Some(value) = self.value {
            params.value = Some(value);
        }
        if let Some(step) = self.value_step {
            params.value_step = Some(step);
        }
        if let Some(discrete) = self.discrete_values {
            params.discrete_values = Some(discrete);
        }

        let slider = parent.add(params);
        if let Some(handler) = self.on_value_changed {
            crate::shared::runtime::register_value(name, handler);
        }
        slider
    }
}
