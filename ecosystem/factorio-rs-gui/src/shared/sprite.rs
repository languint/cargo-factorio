//! Sprite builder.

use factorio_rs::{
    factorio_api::classes::{LuaGuiElement, LuaGuiElementAddParams},
    prelude::*,
};

/// A static sprite (`GuiElementType::Sprite`).
pub struct Sprite {
    path: &'static str,
    name: Option<String>,
    resize: Option<bool>,
}

impl Sprite {
    /// Sprite with the given sprite path (e.g. `"item/iron-plate"`).
    #[must_use]
    pub fn new(path: &'static str) -> Self {
        Self {
            path,
            name: None,
            resize: None,
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
        self.resize = Some(resize);
        self
    }

    /// Create this sprite under `parent`.
    #[must_use]
    pub fn mount(self, parent: LuaGuiElement) -> LuaGuiElement {
        let mut params = LuaGuiElementAddParams {
            r#type: GuiElementType::Sprite,
            sprite: Some(self.path),
            ..Default::default()
        };
        if let Some(name) = self.name {
            params.name = Some(name);
        }
        if let Some(resize) = self.resize {
            params.resize_to_sprite = Some(resize);
        }
        parent.add(params)
    }
}
