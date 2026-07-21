//! Public helpers other mods can `require("__provider__/lua/shared/api")`.

pub const VERSION: i32 = 1;

#[factorio_rs::export]
pub fn greet(name: &str) {
    println!("hello from provider lib, {name}!");
}

/// Hot-path helper: dependents bind via `require` (see `#[factorio_rs::inline]`).
#[factorio_rs::inline]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
