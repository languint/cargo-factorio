//! High-level reactive GUI helpers for factorio-rs mods.
//!
//! Click handlers are stored in the **consuming** mod's `storage`. Forward
//! `OnGuiClick` from your control stage with [`shared::runtime::dispatch_click`]
//! (see the `gui_counter` example). A library-mod event handler cannot see
//! another mod's `storage`.

pub mod shared;

mod factorio_exports;
#[allow(unused_imports)] // empty until control `#[factorio_rs::export]` remotes exist
pub use factorio_exports::*;

pub use shared::prelude::*;
