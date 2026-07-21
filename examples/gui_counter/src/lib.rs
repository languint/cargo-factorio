//! Reactive counter GUI with factorio-rs-gui.

#[factorio_rs::control]
mod control {
    use factorio_rs::{
        factorio_api::{IndexOrName, lua_fn, lua_fn0},
        prelude::*,
        tracing,
    };
    use factorio_rs_gui::shared::button::Button;
    use factorio_rs_gui::shared::frame::Frame;
    use factorio_rs_gui::shared::text::Text;
    use factorio_rs_gui::shared::widget::Widget;

    fn app() -> Widget {
        let count = factorio_rs_gui::state!(0);

        let label = format!("Count: {}", count.get());

        let increment = lua_fn(move |event: OnGuiClickEvent| {
            let _ = event;
            tracing::info!("at increment, count: {}!", count.get());
            count.set(count.get() + 1);
        });

        Frame::new()
            .caption("Counter")
            .auto_center()
            .align_horizontal(LuaStyleHorizontalAlign::Center)
            .align_vertical(LuaStyleVerticalAlign::Center)
            .direction(GuiDirection::Vertical)
            .child(Text::new(&label).as_widget())
            .child(Button::new("Increment").on_click(increment).as_widget())
            .as_widget()
    }

    #[factorio_rs::event(OnPlayerCreated)]
    pub fn on_player_created(event: OnPlayerCreatedEvent) {
        if let Some(player) = game.get_player(IndexOrName::Index(event.player_index)) {
            factorio_rs_gui::shared::runtime::mount(player.gui().screen(), lua_fn0(app));
        }
    }

    // Handlers live in this mod's `storage`; dispatch must run here too.
    // Call via the module path — `use …::dispatch_click` would emit a bad require.
    #[factorio_rs::event(OnGuiClick)]
    pub fn on_gui_click(event: OnGuiClickEvent) {
        factorio_rs_gui::shared::runtime::dispatch_click(event);
    }
}
