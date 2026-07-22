---
title: Authoring macros
description: Write macro_rules! and proc-macro DSLs that factorio-rs can transpile.
---

factorio-rs expands macros with **rustc** before lowering to Lua. You can ship
`macro_rules!` helpers in a mod, or publish a dependency proc-macro crate
(for example a JSX-like `gui!` DSL). This page is for **authors** of those
macros. For the built-in Factorio helpers (`item!`, `export`, ...), see
[Macros and attributes](../../reference/macros/).

## Pipeline

```text
cargo check  ->  rustc -Zunpretty=expanded  ->  frontend lower  ->  Lua
```

1. **Typecheck** - your macros must expand under normal Rust (`cargo check`).
2. **Expand** - the CLI dumps the fully expanded crate (stable toolchains use
   `RUSTC_BOOTSTRAP` for `-Zunpretty=expanded`).
3. **Lower** - the frontend parses that expansion and emits Lua.

What the frontend sees is **ordinary Rust after expansion**, not your macro
tokens. Design for the output, not the sugar.

## What works

| Kind | Where it lives | Notes |
| --- | --- | --- |
| `macro_rules!` | Same crate (any stage module) | Expanded in place; definitions may remain in the dump and are ignored |
| Proc macros (`#[proc_macro]`, ...) | Dependency crates | Dependents `use your_crate::gui` (or prelude re-exports) and invoke normally |
| Built-in `factorio_rs::*` macros | SDK | Dual-path today; also fine after expansion |

Cross-mod **functions** still use `#[factorio_rs::export]` /
`#[factorio_rs::inline]`. Macros are a **compile-time** story: expand in the
consumer, then lower. You do not “export” a macro over Factorio `remote`.

## Golden rule

**Expansion must be [supported Rust](../language/).**

If rustc expands your macro to something the frontend cannot lower (`async`,
unsupported std APIs, exotic patterns, ...), `factorio-rs build` fails at lower
time even though `cargo check` passed.

Prefer expansions that look like code you would write by hand in a factorio-rs
mod: `struct` / `impl`, plain `fn`, `if` / `match`, `println!` / `format!`,
Factorio API calls, closures the frontend already accepts.

## Limitations

### Unsupported Rust in the expansion

Anything outside the language inventory fails after expand. Common traps:

- `async` / `.await`, threads, filesystems, networking
- Heavy generics / trait machinery the frontend does not lower
- Std formatting beyond what `println!` / `format!` support (see
  [Supported Rust](../language/#expression-macros))
- Emitting raw Lua or IR - there is no `factorio_rs::emit_lua!` escape hatch

### `locale!` is special

The `locale!` proc macro only **typechecks** keys. The CLI still scrapes
`locale!` from **unexpanded** sources to write `.cfg` files. A third-party
macro that “looks like” `locale!` will not get that treatment unless you emit
real locale files yourself (for example from a build script).

### Factorio metadata attributes

`#[factorio_rs::export]`, `#[factorio_rs::inline]`, `#[factorio_rs::event]`, and stage
attrs (`#[factorio_rs::control]`, ...) are identity (or near-identity) proc macros.
After expansion they leave durable `__factorio_rs_*` marker consts so the
frontend still sees exports and events. **Do not strip or rename those
markers** if you wrap attributed items in your own macros.

If your macro emits event handlers or exported APIs, expand to normal items
that still carry the factorio-rs attributes (or the markers they produce):

```rust
// Good: keep the attribute on the expanded fn
quote! {
    #[factorio_rs::event(OnSingleplayerInit)]
    pub fn on_singleplayer_init() {
        #body
    }
}
```

### Expanded `println!` / `format!`

rustc turns `println!("hi {name}")` into `_print(format_args!(...))` (and
similar for `format!`). The frontend recognizes those forms. Your macros can
emit either `println!` / `format!` **or** the expanded call shapes; both lower
to `game.print` / string concat. Expanded `println!` templates include a
trailing `\n`.

### Typecheck-only `const _: () = ...`

Macros often emit `const _: () = { let _ = some_expr; };` for compile-time
checks. The frontend skips `const _`. Put real runtime work in functions, not
only in those consts.

### Hygiene and paths

Prefer paths that resolve after expansion in the **consumer** crate
(`factorio_rs::prelude::*`, your crate’s public types). Avoid relying on private
helpers from the macro crate unless they are `pub` and depended on normally.

### Diagnostics

Lower errors are reported against the expanded dump (`<expanded>`), not always
the original macro invocation span. When debugging, expand locally (see
below) and inspect the Rust you actually emit.

## Practical examples

### 1. Same-crate `macro_rules!` helper

Good for small sugar that expands to API you already use:

```rust
#[factorio_rs::control]
mod control {
    use factorio_rs::prelude::*;

    macro_rules! chat {
        ($($arg:tt)*) => {
            println!($($arg)*)
        };
    }

    #[factorio_rs::event(OnSingleplayerInit)]
    pub fn on_singleplayer_init() {
        chat!("mod ready");
    }
}
```

After expand this is ordinary `println!` -> `game.print`.

### 2. Builder-style sugar

Keep the expansion explicit and lowerable:

```rust
macro_rules! label {
    ($parent:expr, $caption:expr) => {{
        $parent.add(LuaGuiElementAddParams {
            r#type: GuiElementType::Label,
            caption: Some($caption.into()),
            ..Default::default()
        })
    }};
}

// usage
let title = label!(frame, "Hello");
```

Avoid expanding to traits, async, or unsupported method chains.

### 3. Dependency proc-macro DSL (sketch)

Package a `proc-macro = true` crate. Dependents depend on it like any Cargo
crate; factorio-rs expands it during their build.

**Macro crate** (`factorio-rs-gui` sketch):

```toml
# factorio-rs-gui/Cargo.toml
[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

```rust
// factorio-rs-gui/src/lib.rs
use proc_macro::TokenStream;

/// Toy DSL: `gui!(frame, "Hello")` -> typed `LuaGuiElementAddParams` add.
#[proc_macro]
pub fn gui(input: TokenStream) -> TokenStream {
    // Parse your DSL (JSX-like tokens, builders, ...), then quote supported Rust.
    // Pseudocode - real parsing omitted:
    //
    // quote! {
    //     {
    //         let __root = #parent.gui().screen().add(LuaGuiElementAddParams {
    //             r#type: GuiElementType::Frame,
    //             caption: Some(#title.into()),
    //             ..Default::default()
    //         });
    //         __root
    //     }
    // }
    input
}
```

**Consumer mod:**

```toml
[dependencies]
factorio-rs = "0.3"
factorio-rs-gui = { path = "../factorio-rs-gui" }
```

```rust
use factorio_rs::prelude::*;
use factorio_rs_gui::gui;

#[factorio_rs::event(OnPlayerCreated)]
pub fn on_player_created(event: OnPlayerCreatedEvent) {
    if let Some(player) = game.get_player(IndexOrName::Index(event.player_index)) {
        let _frame = gui!(player.gui().screen(), title = "Hello factorio-rs");
    }
}
```

Design checklist for a DSL crate:

1. **`cargo check` in a sample factorio-rs mod** that uses the macro.
2. **Inspect expansion** (see Debugging) - confirm the quoted Rust matches the
   language inventory.
3. **Re-export types** the expansion needs (`GuiElementType`, params structs)
   or document `use factorio_rs::prelude::*` in consumers.
4. Prefer **expression macros** that evaluate to values over injecting
   mysterious items at module scope, unless you also emit clean `fn` /
   `struct` forms.

A React-like tree (`gui! { <frame>...</frame> }`) is fine as long as the proc
macro quotes imperative Factorio GUI construction (or other supported code),
not browser DOM APIs.

### 4. Wrapping stage or event items

If your macro generates handlers, leave factorio-rs attributes on the output:

```rust
quote! {
    #[factorio_rs::control]
    mod control {
        #[factorio_rs::event(OnSingleplayerInit)]
        pub fn on_singleplayer_init() {
            #(#stmts)*
        }
    }
}
```

Generating bare `pub fn on_singleplayer_init` **without** `#[factorio_rs::event]`
(and without the expansion markers that attribute produces) will typecheck but
will **not** register as a Factorio event.

## Debugging

### See the expansion rustc produces

```bash
# From the mod crate (same flag the CLI uses on stable):
RUSTC_BOOTSTRAP=1 cargo rustc --profile=check --lib -- -Zunpretty=expanded
```

Or install [cargo-expand](https://github.com/dtolnay/cargo-expand) and run
`cargo expand` (nightly). Diff that output against code you know lowers.

### Iterate with the CLI

```bash
factorio-rs check    # typecheck + expand + lower, no emit
factorio-rs build    # full transpile
```

Lower failures print against `<expanded>`. Fix the **quoted** Rust, not only
the macro parser.

### Keep a golden mod

Maintain a tiny factorio-rs example that uses every public macro form. Run it
in CI with `factorio-rs build` so expansions cannot drift into unsupported Rust.

## See also

- [Macros and attributes](../../reference/macros/) - built-in `factorio_rs::*` surface
- [Supported Rust](../language/) - what expansions may contain
- [Sharing code between mods](../dependencies/) - exporting **functions**, not macros
- [GUI basics](../../recipes/gui-basics/) - imperative GUI without a DSL
