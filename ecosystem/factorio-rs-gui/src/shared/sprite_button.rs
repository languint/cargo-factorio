//! Sprite button builder.

use factorio_rs::{
    factorio_api::{
        LuaFunction,
        classes::{LuaGuiElement, LuaGuiElementAddParams},
    },
    prelude::*,
};

/// A clickable sprite button (`GuiElementType::SpriteButton`).
pub struct SpriteButton {
    sprite: &'static str,
    name: Option<String>,
    clicked_sprite: Option<&'static str>,
    hovered_sprite: Option<&'static str>,
    number: Option<f64>,
    on_click: Option<LuaFunction>,
}

impl SpriteButton {
    /// Sprite button with the given sprite path.
    #[must_use]
    pub fn new(sprite: &'static str) -> Self {
        Self {
            sprite,
            name: None,
            clicked_sprite: None,
            hovered_sprite: None,
            number: None,
            on_click: None,
        }
    }

    /// Optional stable element name (auto-assigned when an `on_click` is set).
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sprite shown while pressed.
    #[must_use]
    pub fn clicked_sprite(mut self, sprite: &'static str) -> Self {
        self.clicked_sprite = Some(sprite);
        self
    }

    /// Sprite shown while hovered.
    #[must_use]
    pub fn hovered_sprite(mut self, sprite: &'static str) -> Self {
        self.hovered_sprite = Some(sprite);
        self
    }

    /// Optional number badge on the button.
    #[must_use]
    pub fn number(mut self, number: f64) -> Self {
        self.number = Some(number);
        self
    }

    /// Register a click handler (routed via the shared click dispatcher).
    #[must_use]
    pub fn on_click(mut self, handler: LuaFunction) -> Self {
        self.on_click = Some(handler);
        self
    }

    /// Create this sprite button under `parent` and register its click handler.
    #[must_use]
    pub fn mount(self, parent: LuaGuiElement) -> LuaGuiElement {
        let name = match self.name {
            Some(name) => name,
            None => {
                if self.on_click.is_some() {
                    crate::shared::runtime::next_element_name("frg_sbtn")
                } else {
                    String::new()
                }
            }
        };

        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::SpriteButton,
            sprite: Some(self.sprite),
            ..Default::default()
        };
        if !name.is_empty() {
            params.name = Some(name.clone());
        }
        if let Some(sprite) = self.clicked_sprite {
            params.clicked_sprite = Some(sprite);
        }
        if let Some(sprite) = self.hovered_sprite {
            params.hovered_sprite = Some(sprite);
        }
        if let Some(number) = self.number {
            params.number = Some(number);
        }

        let button = parent.add(params);
        if let Some(handler) = self.on_click {
            crate::shared::runtime::register_click(name, handler);
        }
        button
    }
}
