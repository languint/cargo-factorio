//! Public helpers other mods can `require("__provider__/lua/shared/api")`.

pub const VERSION: i32 = 1;

#[factorio_rs::export]
pub fn greet(name: &str) {
    println!("hello from provider lib, {name}!");
}
