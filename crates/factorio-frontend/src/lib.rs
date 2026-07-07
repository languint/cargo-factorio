//! Parse a small subset of Rust source code into [`factorio_ir`].

mod error;
mod lower;

pub use error::{FrontendError, FrontendResult};
pub use lower::parse_module;
