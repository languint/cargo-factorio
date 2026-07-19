//! Hand-written prototype stubs for the data stage (`data.extend`).
//!
//! These are not generated from `prototype-api.json` yet. They exist so mods can
//! register common prototypes with typed struct literals; the codegen injects
//! Factorio's `type = "..."` discriminant from the Rust struct name.

/// Minimal [`ItemPrototype`](https://lua-api.factorio.com/latest/prototypes/ItemPrototype.html)
/// for `data.extend`.
///
/// Required Factorio fields: `name`, `icon` (or `icons`), `stack_size`.
/// `type = "item"` is injected by the Lua generator.
///
/// Optional fields omit as Lua `nil` when `None`. Prefer
/// `..Default::default()` for fields you do not set (same sparse-table pattern
/// as other API structs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Item {
    /// Internal prototype name (e.g. `"my-mod-widget"`).
    pub name: &'static str,
    /// Packaged icon path (e.g. `"__my_mod__/graphics/icon.png"`).
    pub icon: &'static str,
    /// Max items per inventory slot.
    pub stack_size: i64,
    /// Icon pixel size. Factorio defaults to `64` when omitted.
    pub icon_size: Option<i64>,
    /// Item subgroup id (e.g. `"intermediate-product"`).
    pub subgroup: Option<&'static str>,
    /// Sort order within the subgroup.
    pub order: Option<&'static str>,
}
