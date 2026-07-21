//! Root widget enum and mounting.

use factorio_rs::factorio_api::classes::LuaGuiElement;

// `crate::` paths (not `super::`) so cross-module `impl` methods attach to the
// imported Lua tables — `use super::…` is not lowered to a require.
use crate::shared::button::Button;
use crate::shared::frame::Frame;
use crate::shared::text::Text;

/// A mountable GUI node.
pub enum Widget {
    Frame(Frame),
    Text(Text),
    Button(Button),
}

impl Widget {
    /// Wrap a [`Frame`].
    #[must_use]
    pub fn from_frame(frame: Frame) -> Self {
        Self::Frame(frame)
    }

    /// Wrap a [`Text`] label.
    #[must_use]
    pub fn from_text(text: Text) -> Self {
        Self::Text(text)
    }

    /// Wrap a [`Button`].
    #[must_use]
    pub fn from_button(button: Button) -> Self {
        Self::Button(button)
    }

    /// Create Factorio GUI elements under `parent`.
    pub fn mount(self, parent: LuaGuiElement) {
        match self {
            Self::Frame(frame) => frame.mount(parent),
            Self::Text(text) => {
                let _ = text.mount(parent);
            }
            Self::Button(button) => {
                let _ = button.mount(parent);
            }
        }
    }
}

impl Frame {
    /// Convenience: append this frame as a [`Widget`].
    #[must_use]
    pub fn as_widget(self) -> Widget {
        Widget::from_frame(self)
    }
}

impl Text {
    /// Convenience: append this label as a [`Widget`].
    #[must_use]
    pub fn as_widget(self) -> Widget {
        Widget::from_text(self)
    }
}

impl Button {
    /// Convenience: append this button as a [`Widget`].
    #[must_use]
    pub fn as_widget(self) -> Widget {
        Widget::from_button(self)
    }
}
