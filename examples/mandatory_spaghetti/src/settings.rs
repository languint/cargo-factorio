use factorio_rs::prelude::*;

factorio_rs::mod_settings! {
    prefix = "msr",

    startup {
        casual_mode: bool = false,
        adjacency_enabled: bool = true,
    }
}
