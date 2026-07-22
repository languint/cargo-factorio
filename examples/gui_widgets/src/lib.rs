#[factorio_rs::control]
mod control {
    use factorio_rs::{
        factorio_api::{IndexOrName, lua_fn, lua_fn0},
        prelude::*,
    };

    use factorio_rs_gui::shared::button::Button;
    use factorio_rs_gui::shared::checkbox::Checkbox;
    use factorio_rs_gui::shared::flow::Flow;
    use factorio_rs_gui::shared::frame::Frame;
    use factorio_rs_gui::shared::line::Line;
    use factorio_rs_gui::shared::progress_bar::ProgressBar;
    use factorio_rs_gui::shared::slider::Slider;
    use factorio_rs_gui::shared::text::Text;
    use factorio_rs_gui::shared::text_field::TextField;
    use factorio_rs_gui::shared::widget::Widget;

    const ROOT: &str = "gui_widgets";

    fn app() -> impl Into<Widget> {
        let checks = factorio_rs_gui::state!(0);
        let slides = factorio_rs_gui::state!(0);
        let texts = factorio_rs_gui::state!(0);

        let status = format!(
            "checks={} slides={} texts={}",
            checks.get(),
            slides.get(),
            texts.get()
        );

        let on_checked = lua_fn(move |event: OnGuiCheckedStateChangedEvent| {
            let _ = event;
            checks.set(checks.get() + 1);
        });
        let on_slide = lua_fn(move |event: OnGuiValueChangedEvent| {
            let _ = event;
            slides.set(slides.get() + 1);
        });
        let on_text = lua_fn(move |event: OnGuiTextChangedEvent| {
            let _ = event;
            texts.set(texts.get() + 1);
        });

        Frame::new()
            .caption("Widget smoke")
            .centered()
            .direction(GuiDirection::Vertical)
            .child(Text::new(&status))
            .child(Line::new().direction(GuiDirection::Horizontal))
            .child(
                Flow::new()
                    .direction(GuiDirection::Vertical)
                    .child(Checkbox::new("Enable").state(false).on_checked(on_checked))
                    .child(
                        Slider::new()
                            .minimum_value(0.0)
                            .maximum_value(10.0)
                            .value(3.0)
                            .value_step(1.0)
                            .discrete_values(true)
                            .on_value_changed(on_slide),
                    )
                    .child(TextField::new().text("edit me").on_text_changed(on_text))
                    .child(ProgressBar::new().value(0.4).caption("40%"))
                    .child(Button::new("No-op")),
            )
    }

    #[factorio_rs::event(OnPlayerCreated)]
    pub fn on_player_created(event: OnPlayerCreatedEvent) {
        if let Some(player) = game.get_player(IndexOrName::Index(event.player_index)) {
            factorio_rs_gui::shared::runtime::mount(player.gui().screen(), ROOT, lua_fn0(app));
        }
    }

    #[factorio_rs::event(OnTick)]
    pub fn on_tick(_event: OnTickEvent) {
        factorio_rs_gui::shared::runtime::install(ROOT, lua_fn0(app));
    }
}
