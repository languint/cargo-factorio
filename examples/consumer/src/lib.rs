#[factorio_rs::control]
mod control {
    use provider::shared::api;

    #[factorio_rs::event(OnSingleplayerInit)]
    pub fn on_singleplayer_init() {
        provider::greet("consumer");
        api::greet("consumer");
        println!("provider API version: {}", api::VERSION);
    }
}
