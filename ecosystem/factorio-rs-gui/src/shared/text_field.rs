//! Text field builder.

use factorio_rs::{
    factorio_api::{
        LuaFunction,
        classes::{LuaGuiElement, LuaGuiElementAddParams},
    },
    prelude::*,
};

/// A single-line text field (`GuiElementType::Textfield`).
pub struct TextField {
    text: Option<String>,
    name: Option<String>,
    numeric: Option<bool>,
    allow_decimal: Option<bool>,
    allow_negative: Option<bool>,
    is_password: Option<bool>,
    lose_focus_on_confirm: Option<bool>,
    on_text_changed: Option<LuaFunction>,
    on_confirmed: Option<LuaFunction>,
}

impl TextField {
    /// Start a new text-field builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            text: None,
            name: None,
            numeric: None,
            allow_decimal: None,
            allow_negative: None,
            is_password: None,
            lose_focus_on_confirm: None,
            on_text_changed: None,
            on_confirmed: None,
        }
    }

    /// Initial text contents.
    #[must_use]
    pub fn text(mut self, text: &str) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Optional stable element name (auto-assigned when a handler is set).
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Restrict input to numeric characters.
    #[must_use]
    pub fn numeric(mut self, numeric: bool) -> Self {
        self.numeric = Some(numeric);
        self
    }

    /// Allow a decimal point when numeric.
    #[must_use]
    pub fn allow_decimal(mut self, allow: bool) -> Self {
        self.allow_decimal = Some(allow);
        self
    }

    /// Allow a leading minus when numeric.
    #[must_use]
    pub fn allow_negative(mut self, allow: bool) -> Self {
        self.allow_negative = Some(allow);
        self
    }

    /// Mask input as a password field.
    #[must_use]
    pub fn is_password(mut self, password: bool) -> Self {
        self.is_password = Some(password);
        self
    }

    /// Lose focus when the player confirms (Enter).
    #[must_use]
    pub fn lose_focus_on_confirm(mut self, lose: bool) -> Self {
        self.lose_focus_on_confirm = Some(lose);
        self
    }

    /// Register a text-changed handler.
    #[must_use]
    pub fn on_text_changed(mut self, handler: LuaFunction) -> Self {
        self.on_text_changed = Some(handler);
        self
    }

    /// Register a confirmed (Enter) handler.
    #[must_use]
    pub fn on_confirmed(mut self, handler: LuaFunction) -> Self {
        self.on_confirmed = Some(handler);
        self
    }

    /// Create this text field under `parent` and register handlers.
    #[must_use]
    pub fn mount(self, parent: LuaGuiElement) -> LuaGuiElement {
        let needs_name = self.on_text_changed.is_some() || self.on_confirmed.is_some();
        let name = match self.name {
            Some(name) => name,
            None => {
                if needs_name {
                    crate::shared::runtime::next_element_name("frg_tf")
                } else {
                    String::new()
                }
            }
        };

        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::Textfield,
            ..Default::default()
        };
        if !name.is_empty() {
            params.name = Some(name.clone());
        }
        if let Some(text) = self.text {
            params.text = Some(text);
        }
        if let Some(numeric) = self.numeric {
            params.numeric = Some(numeric);
        }
        if let Some(allow) = self.allow_decimal {
            params.allow_decimal = Some(allow);
        }
        if let Some(allow) = self.allow_negative {
            params.allow_negative = Some(allow);
        }
        if let Some(password) = self.is_password {
            params.is_password = Some(password);
        }
        if let Some(lose) = self.lose_focus_on_confirm {
            params.lose_focus_on_confirm = Some(lose);
        }

        let field = parent.add(params);
        if let Some(handler) = self.on_text_changed {
            crate::shared::runtime::register_text(name.clone(), handler);
        }
        if let Some(handler) = self.on_confirmed {
            crate::shared::runtime::register_confirmed(name, handler);
        }
        field
    }
}
