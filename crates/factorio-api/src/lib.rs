//! Generated Factorio runtime API bindings.
//!
//! These types exist for Rust type-checking and IDE support. Mod code is
//! transpiled to Lua and never executes these stub implementations.

#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    non_upper_case_globals,
    clippy::all,
    clippy::pedantic,
    clippy::nursery
)]

/// Opaque placeholder for complex Factorio Lua API values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LuaAny;

include!(concat!(env!("OUT_DIR"), "/mod.rs"));

pub use event_filters::{EventFilterEntry, FilterMethodSpec, filter_method_spec};
pub use map::{event_filter_type, event_type_to_name};

pub mod prelude {
    pub use crate::event_data::*;
    pub use crate::event_filters::*;
    pub use crate::event_type_to_name;
    pub use crate::events::*;
    pub use crate::globals::*;
}
