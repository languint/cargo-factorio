//! Root widget enum and mounting.

use factorio_rs::factorio_api::classes::LuaGuiElement;

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

    /// Apply `root_name` to a root [`Frame`] when the builder left `name` unset.
    #[must_use]
    pub fn with_root_name(self, root_name: &str) -> Self {
        match self {
            Self::Frame(frame) => Self::Frame(frame.ensure_name(root_name)),
            other => other,
        }
    }
}

impl From<Frame> for Widget {
    fn from(value: Frame) -> Self {
        Self::Frame(value)
    }
}

impl From<Text> for Widget {
    fn from(value: Text) -> Self {
        Self::Text(value)
    }
}

impl From<Button> for Widget {
    fn from(value: Button) -> Self {
        Self::Button(value)
    }
}
