//! Progress bar builder.

use factorio_rs::{
    factorio_api::classes::{LuaGuiElement, LuaGuiElementAddParams},
    prelude::*,
};

/// A progress bar (`GuiElementType::Progressbar`).
pub struct ProgressBar {
    name: Option<String>,
    value: Option<f64>,
    caption: Option<String>,
}

impl ProgressBar {
    /// Start a new progress-bar builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            value: None,
            caption: None,
        }
    }

    /// Optional stable element name.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Progress in `[0.0, 1.0]`.
    #[must_use]
    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }

    /// Optional caption overlaid on the bar.
    #[must_use]
    pub fn caption(mut self, caption: &str) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Create this progress bar under `parent`.
    #[must_use]
    pub fn mount(self, parent: LuaGuiElement) -> LuaGuiElement {
        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::Progressbar,
            ..Default::default()
        };
        if let Some(name) = self.name {
            params.name = Some(name);
        }
        if let Some(value) = self.value {
            params.value = Some(value);
        }
        if let Some(caption) = self.caption {
            params.caption = Some(caption);
        }
        parent.add(params)
    }
}
