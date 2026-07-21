//! Shared-stage GUI helpers (loaded by dependents via `require`).

#![allow(
    clippy::missing_const_for_fn,
    clippy::single_match,
    clippy::manual_unwrap_or_default,
    clippy::option_if_let_else,
    clippy::unwrap_or_default,
    clippy::new_without_default
)]

pub mod align;
pub mod button;
pub mod frame;
pub mod prelude;
pub mod runtime;
pub mod state;
pub mod text;
pub mod widget;
