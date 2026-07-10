//! Rust SDK for Factorio's modding API.

pub use factorio_api::{self, event_type_to_name};
pub use factorio_macros::{control, data, event, mod_settings, settings, shared};
pub use factorio_macros::{control_mod, data_mod, settings_mod, shared_mod};

pub mod prelude {
    pub use crate::{
        control, control_mod, data, data_mod, event, mod_settings, settings, settings_mod, shared,
        shared_mod,
    };
    pub use factorio_api::prelude::*;
}
