#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    dead_code
)]

use factorio_frontend::FrontendResult;
use factorio_ir::module::Module;

#[track_caller]
pub fn must_ok_parse(result: FrontendResult<Module>) -> Module {
    result.unwrap_or_else(|error| panic!("parse_module failed: {error:?}"))
}

#[track_caller]
pub fn must_ok<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    result.unwrap_or_else(|error| panic!("expected Ok, got Err({error:?})"))
}
