---
title: locale_test
description: Console command that prints a localized greeting with the player name.
---

Path: `examples/locale_test`.

Registers `/greet <1-3>`, picks a greeting locale key from a constant array, and
prints a Factorio `LocalisedString` that interpolates the player's name
(`__1__`). Translations ship for English, German, and Spanish via `locale!`.

```rust
const GREETINGS: &[&str] = &[
    "greetings.hello",
    "greetings.welcome",
    "greetings.howdy",
];

factorio_rs::locale! {
    en {
        greetings {
            "hello" = "Hello, __1__!",
            // ...
        }
    }
}

commands.add_command("greet", ["greetings.command-help"], lua_fn(greet));

// Keep the handler non-`pub` so it lowers to `local function greet` (callable by name).
fn greet(command: CustomCommandData) {
    // ...
    player.print([key, player.name()], None);
}
// → player.print({ key, player.name })
```

`lua_fn` coerces a Rust `fn` item to `LuaFunction` for `cargo check` and is
stripped when lowering (Lua still receives the function name).

## Try it

```bash
cd examples/locale_test
factorio-rs build
factorio-rs install --open   # optional
```

In-game (change language under Settings → Interface to see other locales):

```text
/greet 1
/greet 2
/greet 3
```

See [Locale](../guides/locale/) for `locale!` and runtime `LocalisedString` usage.
