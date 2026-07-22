//! Root widget enum and mounting.

use factorio_rs::factorio_api::classes::LuaGuiElement;

use crate::shared::button::Button;
use crate::shared::checkbox::Checkbox;
use crate::shared::drop_down::DropDown;
use crate::shared::empty_widget::EmptyWidget;
use crate::shared::flow::Flow;
use crate::shared::frame::Frame;
use crate::shared::line::Line;
use crate::shared::progress_bar::ProgressBar;
use crate::shared::scroll_pane::ScrollPane;
use crate::shared::slider::Slider;
use crate::shared::sprite::Sprite;
use crate::shared::sprite_button::SpriteButton;
use crate::shared::text::Text;
use crate::shared::text_field::TextField;

/// A mountable GUI node.
pub enum Widget {
    Frame(Frame),
    Flow(Flow),
    ScrollPane(ScrollPane),
    Text(Text),
    Button(Button),
    SpriteButton(SpriteButton),
    Checkbox(Checkbox),
    TextField(TextField),
    Slider(Slider),
    ProgressBar(ProgressBar),
    DropDown(DropDown),
    Sprite(Sprite),
    Line(Line),
    EmptyWidget(EmptyWidget),
}

impl Widget {
    /// Create Factorio GUI elements under `parent`.
    pub fn mount(self, parent: LuaGuiElement) {
        match self {
            Self::Frame(frame) => frame.mount(parent),
            Self::Flow(flow) => flow.mount(parent),
            Self::ScrollPane(pane) => pane.mount(parent),
            Self::Text(text) => {
                let _ = text.mount(parent);
            }
            Self::Button(button) => {
                let _ = button.mount(parent);
            }
            Self::SpriteButton(button) => {
                let _ = button.mount(parent);
            }
            Self::Checkbox(checkbox) => {
                let _ = checkbox.mount(parent);
            }
            Self::TextField(field) => {
                let _ = field.mount(parent);
            }
            Self::Slider(slider) => {
                let _ = slider.mount(parent);
            }
            Self::ProgressBar(bar) => {
                let _ = bar.mount(parent);
            }
            Self::DropDown(dropdown) => {
                let _ = dropdown.mount(parent);
            }
            Self::Sprite(sprite) => {
                let _ = sprite.mount(parent);
            }
            Self::Line(line) => {
                let _ = line.mount(parent);
            }
            Self::EmptyWidget(empty) => {
                let _ = empty.mount(parent);
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

impl From<Flow> for Widget {
    fn from(value: Flow) -> Self {
        Self::Flow(value)
    }
}

impl From<ScrollPane> for Widget {
    fn from(value: ScrollPane) -> Self {
        Self::ScrollPane(value)
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

impl From<SpriteButton> for Widget {
    fn from(value: SpriteButton) -> Self {
        Self::SpriteButton(value)
    }
}

impl From<Checkbox> for Widget {
    fn from(value: Checkbox) -> Self {
        Self::Checkbox(value)
    }
}

impl From<TextField> for Widget {
    fn from(value: TextField) -> Self {
        Self::TextField(value)
    }
}

impl From<Slider> for Widget {
    fn from(value: Slider) -> Self {
        Self::Slider(value)
    }
}

impl From<ProgressBar> for Widget {
    fn from(value: ProgressBar) -> Self {
        Self::ProgressBar(value)
    }
}

impl From<DropDown> for Widget {
    fn from(value: DropDown) -> Self {
        Self::DropDown(value)
    }
}

impl From<Sprite> for Widget {
    fn from(value: Sprite) -> Self {
        Self::Sprite(value)
    }
}

impl From<Line> for Widget {
    fn from(value: Line) -> Self {
        Self::Line(value)
    }
}

impl From<EmptyWidget> for Widget {
    fn from(value: EmptyWidget) -> Self {
        Self::EmptyWidget(value)
    }
}
