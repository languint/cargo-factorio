#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    dead_code
)]

use factorio_codegen::LuaGeneratorError;

#[track_caller]
pub fn must_ok(result: Result<String, LuaGeneratorError>) -> String {
    result.unwrap_or_else(|error| panic!("generate_module failed: {error:?}"))
}

#[track_caller]
pub fn must_err(result: Result<String, LuaGeneratorError>) -> LuaGeneratorError {
    result.unwrap_err()
}
