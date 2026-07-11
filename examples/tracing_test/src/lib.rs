factorio_rs::control_mod! {
    use factorio_rs::tracing;

    #[factorio_rs::event(OnSingleplayerInit)]
    pub fn on_singleplayer_init() {
        tracing::info!("Hello factorio-rs!");
        tracing::error!("Oopsies!");
    }
}
