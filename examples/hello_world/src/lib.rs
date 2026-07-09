#[factorio_rs::control]
mod control {
    use factorio_rs::factorio_api::events::{OnBuiltEntityEvent, OnBuiltEntityFilter};

    #[factorio_rs::event(OnSingleplayerInit)]
    pub fn on_singleplayer_init() {
        println!("Hello factorio-rs!");
    }

    #[factorio_rs::event(filter = OnBuiltEntityFilter::name("inserter"))]
    pub fn on_built_entity(event: OnBuiltEntityEvent) {
        println!("inserter built at: {:?}", event.entity.position);
        event.entity.destroy(None);
    }
}
