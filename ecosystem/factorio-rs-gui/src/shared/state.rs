//! Hook state re-exports and the [`state!`] macro.

pub use super::runtime::State;

/// Create reactive state that survives rebuilds (hook-ordered).
///
/// Expands to [`State::use_state`](crate::shared::runtime::State::use_state).
#[macro_export]
macro_rules! state {
    ($init:expr) => {
        $crate::shared::runtime::State::use_state($init)
    };
}
