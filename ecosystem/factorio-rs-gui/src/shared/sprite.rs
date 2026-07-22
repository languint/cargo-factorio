//! Sprite builder.

use factorio_rs::{
    factorio_api::classes::{LuaGuiElement, LuaGuiElementAddParams},
    prelude::*,
};

/// A static sprite (`GuiElementType::Sprite`).
pub struct Sprite {
    sprite: &'static str,
    name: Option<String>,
    resize_to_sprite: Option<bool>,
}

impl Sprite {
    /// Sprite with the given sprite path (e.g. `"item/iron-plate"`).
    #[must_use]
    pub fn new(sprite: &'static str) -> Self {
        Self {
            sprite,
            name: None,
            resize_to_sprite: None,
        }
    }

    /// Optional stable element name.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Size the element to the sprite.
    #[must_use]
    pub fn resize_to_sprite(mut self, resize: bool) -> Self {
        self.resize_to_sprite = Some(resize);
        self
    }

    /// Create this sprite under `parent`.
    #[must_use]
    pub fn mount(self, parent: LuaGuiElement) -> LuaGuiElement {
        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::Sprite,
            sprite: Some(self.sprite),
            ..Default::default()
        };
        if let Some(name) = self.name {
            params.name = Some(name);
        }
        if let Some(resize) = self.resize_to_sprite {
            params.resize_to_sprite = Some(resize);
        }
        parent.add(params)
    }
}
