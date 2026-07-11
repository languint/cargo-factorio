use factorio_rs::prelude::*;

factorio_rs::mod_settings! {
    prefix = "msr",

    startup {
        casual_mode: bool = false,
        adjacency_enabled: bool = true,
    }
}

factorio_rs::locale! {
    file = "settings",

    en {
        mod_setting_name {
            Settings::CASUAL_MODE = "Casual mode",
            Settings::ADJACENCY_ENABLED = "Enable adjacency restriction",
        }
        mod_setting_description {
            Settings::CASUAL_MODE = "Entities drop on the ground instead of getting destroyed.",
        }
    }

    de {
        mod_setting_name {
            Settings::CASUAL_MODE = "Lässig Modus",
            Settings::ADJACENCY_ENABLED = "Nachbarschaftsbeschränkung aktivieren",
        }
        mod_setting_description {
            Settings::CASUAL_MODE = "Entitäten fallen zu Boden, anstatt zerstört zu werden.",
        }
    }
}
