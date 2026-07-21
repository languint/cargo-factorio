#[factorio_rs::control]
mod control {
    use provider::shared::api;

    #[factorio_rs::event(OnSingleplayerInit)]
    pub fn on_singleplayer_init() {
        provider::greet("consumer");
        api::greet("consumer");
        let _sum = api::add(2, 3);
        println!("provider API version: {}", api::VERSION);
    }
}
