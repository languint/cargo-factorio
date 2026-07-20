//! Allowlisted data-stage prototype stubs from `prototype-api.json`.
//!
//! Factorio prototype properties are mostly complex aliases (`FileName`,
//! `ItemCountType`, nested tables). Mid-mod stubs use a **curated** field set
//! per allowlist entry, validated against the JSON inheritance chain, so we keep
//! the sparse `Default` pattern of the former hand-written stubs.

use std::collections::{BTreeMap, HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::quote;

use crate::generate::ident::{make_ident, sanitize_doc};
use crate::schema_prototype::{PrototypeApi, PrototypeDef, PrototypeProperty};

/// `(Factorio typename, Rust struct name)` pairs generated for `data.extend`.
pub const PROTOTYPE_ALLOWLIST: &[(&str, &str)] = &[
    ("item", "Item"),
    ("recipe", "Recipe"),
    ("technology", "Technology"),
    ("fluid", "Fluid"),
    ("assembling-machine", "AssemblingMachine"),
];

#[derive(Clone, Copy)]
enum FieldKind {
    Str,
    I64,
    F64,
    OptStr,
    OptI64,
    OptF64,
    OptBool,
    StrSlice,
    OptStrSlice,
    RecipeIngredients,
    RecipeProducts,
    TechPrereqs,
    TechEffects,
    TechUnit,
    Color,
    BoundingBox,
    EnergySource,
    Minable,
}

struct FieldSpec {
    name: &'static str,
    kind: FieldKind,
    doc: &'static str,
}

fn fields_for(rust_name: &str) -> &'static [FieldSpec] {
    match rust_name {
        "Item" => &[
            FieldSpec {
                name: "name",
                kind: FieldKind::Str,
                doc: "Internal prototype name (e.g. `\"my-mod-widget\"`).",
            },
            FieldSpec {
                name: "icon",
                kind: FieldKind::Str,
                doc: "Packaged icon path (e.g. `\"__my_mod__/graphics/icon.png\"`).",
            },
            FieldSpec {
                name: "stack_size",
                kind: FieldKind::I64,
                doc: "Max items per inventory slot.",
            },
            FieldSpec {
                name: "icon_size",
                kind: FieldKind::OptI64,
                doc: "Icon pixel size. Factorio defaults to `64` when omitted.",
            },
            FieldSpec {
                name: "subgroup",
                kind: FieldKind::OptStr,
                doc: "Item subgroup id (e.g. `\"intermediate-product\"`).",
            },
            FieldSpec {
                name: "order",
                kind: FieldKind::OptStr,
                doc: "Sort order within the subgroup.",
            },
        ],
        "Recipe" => &[
            FieldSpec {
                name: "name",
                kind: FieldKind::Str,
                doc: "Internal prototype name (e.g. `\"my-mod-widget\"`).",
            },
            FieldSpec {
                name: "ingredients",
                kind: FieldKind::RecipeIngredients,
                doc: "Crafting ingredients.",
            },
            FieldSpec {
                name: "results",
                kind: FieldKind::RecipeProducts,
                doc: "Crafting products.",
            },
            FieldSpec {
                name: "energy_required",
                kind: FieldKind::OptF64,
                doc: "Crafting energy in seconds. Factorio defaults to `0.5` when omitted.",
            },
            FieldSpec {
                name: "category",
                kind: FieldKind::OptStr,
                doc: "Recipe category id (e.g. `\"crafting\"`). Emitted as Lua `category`.",
            },
            FieldSpec {
                name: "enabled",
                kind: FieldKind::OptBool,
                doc: "Whether the recipe is unlocked at start.",
            },
            FieldSpec {
                name: "subgroup",
                kind: FieldKind::OptStr,
                doc: "Item subgroup id.",
            },
            FieldSpec {
                name: "order",
                kind: FieldKind::OptStr,
                doc: "Sort order within the subgroup.",
            },
        ],
        "Technology" => &[
            FieldSpec {
                name: "name",
                kind: FieldKind::Str,
                doc: "Internal prototype name (e.g. `\"my-mod-widget\"`).",
            },
            FieldSpec {
                name: "icon",
                kind: FieldKind::Str,
                doc: "Packaged tech icon path.",
            },
            FieldSpec {
                name: "icon_size",
                kind: FieldKind::OptI64,
                doc: "Icon pixel size. Technology icons are often `256`.",
            },
            FieldSpec {
                name: "prerequisites",
                kind: FieldKind::TechPrereqs,
                doc: "Prerequisite technology ids.",
            },
            FieldSpec {
                name: "effects",
                kind: FieldKind::TechEffects,
                doc: "Effects applied on research (typically unlock-recipe).",
            },
            FieldSpec {
                name: "unit",
                kind: FieldKind::TechUnit,
                doc: "Lab cost.",
            },
            FieldSpec {
                name: "order",
                kind: FieldKind::OptStr,
                doc: "Sort order string.",
            },
        ],
        "Fluid" => &[
            FieldSpec {
                name: "name",
                kind: FieldKind::Str,
                doc: "Internal fluid prototype name.",
            },
            FieldSpec {
                name: "icon",
                kind: FieldKind::Str,
                doc: "Packaged icon path.",
            },
            FieldSpec {
                name: "default_temperature",
                kind: FieldKind::F64,
                doc: "Default temperature of the fluid.",
            },
            FieldSpec {
                name: "base_color",
                kind: FieldKind::Color,
                doc: "Primary fluid color.",
            },
            FieldSpec {
                name: "flow_color",
                kind: FieldKind::Color,
                doc: "Flow / animation color.",
            },
            FieldSpec {
                name: "icon_size",
                kind: FieldKind::OptI64,
                doc: "Icon pixel size.",
            },
            FieldSpec {
                name: "subgroup",
                kind: FieldKind::OptStr,
                doc: "Item subgroup id.",
            },
            FieldSpec {
                name: "order",
                kind: FieldKind::OptStr,
                doc: "Sort order string.",
            },
            FieldSpec {
                name: "hidden",
                kind: FieldKind::OptBool,
                doc: "Hide from factoriopedia / lists when true.",
            },
        ],
        "AssemblingMachine" => &[
            FieldSpec {
                name: "name",
                kind: FieldKind::Str,
                doc: "Internal entity prototype name.",
            },
            FieldSpec {
                name: "icon",
                kind: FieldKind::Str,
                doc: "Packaged icon path.",
            },
            FieldSpec {
                name: "crafting_speed",
                kind: FieldKind::F64,
                doc: "Crafting speed multiplier.",
            },
            FieldSpec {
                name: "crafting_categories",
                kind: FieldKind::StrSlice,
                doc: "Recipe category ids this machine accepts.",
            },
            FieldSpec {
                name: "energy_usage",
                kind: FieldKind::Str,
                doc: "Energy usage string (e.g. `\"150kW\"`).",
            },
            FieldSpec {
                name: "energy_source",
                kind: FieldKind::EnergySource,
                doc: "Simplified energy source table.",
            },
            FieldSpec {
                name: "icon_size",
                kind: FieldKind::OptI64,
                doc: "Icon pixel size.",
            },
            FieldSpec {
                name: "flags",
                kind: FieldKind::OptStrSlice,
                doc: "Entity flags (e.g. `placeable-neutral`, `player-creation`).",
            },
            FieldSpec {
                name: "minable",
                kind: FieldKind::Minable,
                doc: "Mining properties when the entity is mined.",
            },
            FieldSpec {
                name: "max_health",
                kind: FieldKind::OptF64,
                doc: "Maximum health.",
            },
            FieldSpec {
                name: "collision_box",
                kind: FieldKind::BoundingBox,
                doc: "Collision box.",
            },
            FieldSpec {
                name: "selection_box",
                kind: FieldKind::BoundingBox,
                doc: "Selection box.",
            },
            FieldSpec {
                name: "module_slots",
                kind: FieldKind::OptI64,
                doc: "Number of module slots.",
            },
            FieldSpec {
                name: "subgroup",
                kind: FieldKind::OptStr,
                doc: "Item subgroup id.",
            },
            FieldSpec {
                name: "order",
                kind: FieldKind::OptStr,
                doc: "Sort order string.",
            },
        ],
        _ => &[],
    }
}

/// JSON property name used for validation (Recipe `category` is curated-only).
fn json_property_name<'a>(rust_struct: &str, field: &'a str) -> Option<&'a str> {
    match (rust_struct, field) {
        ("Recipe", "category") => None, // curated mid-mod alias; JSON uses `categories`
        _ => Some(field),
    }
}

fn rust_type_tokens(kind: FieldKind) -> TokenStream {
    match kind {
        FieldKind::Str => quote! { &'static str },
        FieldKind::I64 => quote! { i64 },
        FieldKind::F64 => quote! { f64 },
        FieldKind::OptStr => quote! { Option<&'static str> },
        FieldKind::OptI64 => quote! { Option<i64> },
        FieldKind::OptF64 => quote! { Option<f64> },
        FieldKind::OptBool => quote! { Option<bool> },
        FieldKind::StrSlice => quote! { &'static [&'static str] },
        FieldKind::OptStrSlice => quote! { Option<&'static [&'static str]> },
        FieldKind::RecipeIngredients => quote! { &'static [crate::prototypes::RecipeIngredient] },
        FieldKind::RecipeProducts => quote! { &'static [crate::prototypes::RecipeProduct] },
        FieldKind::TechPrereqs => quote! { &'static [&'static str] },
        FieldKind::TechEffects => quote! { &'static [crate::prototypes::UnlockRecipeEffect] },
        FieldKind::TechUnit => quote! { crate::prototypes::TechnologyUnit },
        FieldKind::Color => quote! { crate::prototypes::Color },
        FieldKind::BoundingBox => quote! { Option<crate::prototypes::BoundingBox> },
        FieldKind::EnergySource => quote! { crate::prototypes::EnergySource },
        FieldKind::Minable => quote! { Option<crate::prototypes::MinableProperties> },
    }
}

fn needs_eq(kind: FieldKind) -> bool {
    !matches!(
        kind,
        FieldKind::F64
            | FieldKind::OptF64
            | FieldKind::TechUnit
            | FieldKind::Color
            | FieldKind::BoundingBox
            | FieldKind::EnergySource
            | FieldKind::Minable
            | FieldKind::RecipeIngredients
            | FieldKind::RecipeProducts
            | FieldKind::TechEffects
    )
}

pub fn generate_prototypes(api: &PrototypeApi) -> Result<String, String> {
    let by_name: HashMap<&str, &PrototypeDef> = api
        .prototypes
        .iter()
        .map(|p| (p.name.as_str(), p))
        .collect();

    let mut typename_index: BTreeMap<&str, &PrototypeDef> = BTreeMap::new();
    for proto in &api.prototypes {
        if let Some(typename) = proto.typename.as_deref() {
            typename_index.insert(typename, proto);
        }
    }

    let mut structs = Vec::new();
    for &(typename, rust_name) in PROTOTYPE_ALLOWLIST {
        let Some(proto) = typename_index.get(typename).copied() else {
            return Err(format!(
                "prototype-api.json missing typename `{typename}`"
            ));
        };
        let inherited = collect_properties(proto, &by_name);
        let prop_names: HashSet<&str> = inherited.iter().map(|(n, _)| n.as_str()).collect();

        for field in fields_for(rust_name) {
            if let Some(json_name) = json_property_name(rust_name, field.name)
                && !prop_names.contains(json_name)
            {
                return Err(format!(
                    "allowlist field `{}.{}` not found on {} parent chain",
                    rust_name, field.name, proto.name
                ));
            }
        }

        structs.push(emit_struct(rust_name, typename, proto, fields_for(rust_name)));
    }

    let header = format!(
        "// Generated from Factorio prototype API v{} (format v{}).\n\
         // Allowlisted sparse stubs for data.extend. Companions live in prototypes.rs.\n\
         #[allow(unused, clippy::all, clippy::pedantic, clippy::nursery)]\n\n",
        api.application_version, api.api_version
    );

    Ok(format!(
        "{header}{}",
        structs.into_iter().collect::<TokenStream>()
    ))
}

fn collect_properties<'a>(
    proto: &'a PrototypeDef,
    by_name: &HashMap<&'a str, &'a PrototypeDef>,
) -> Vec<(String, &'a PrototypeProperty)> {
    let mut chain = Vec::new();
    let mut current = Some(proto);
    while let Some(p) = current {
        chain.push(p);
        current = p
            .parent
            .as_deref()
            .and_then(|parent| by_name.get(parent).copied());
    }
    chain.reverse();

    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for ancestor in chain {
        for prop in &ancestor.properties {
            if seen.insert(prop.name.as_str()) {
                out.push((prop.name.clone(), prop));
            }
        }
    }
    out
}

fn emit_struct(
    rust_name: &str,
    typename: &str,
    proto: &PrototypeDef,
    fields: &[FieldSpec],
) -> TokenStream {
    let ident = make_ident(rust_name);
    let link = format!(
        "Minimal [`{}`](https://lua-api.factorio.com/latest/prototypes/{}.html) for `data.extend`.",
        proto.name, proto.name
    );
    let type_doc = format!("`type = \"{typename}\"` is injected by the Lua generator.");
    let desc = sanitize_doc(&proto.description);
    let doc = if desc.is_empty() {
        format!("{link}\n\n{type_doc}")
    } else {
        format!("{link}\n\n{desc}\n\n{type_doc}")
    };

    let mut all_eq = true;
    let field_tokens: Vec<_> = fields
        .iter()
        .map(|field| {
            if !needs_eq(field.kind) {
                all_eq = false;
            }
            let name = make_ident(field.name);
            let ty = rust_type_tokens(field.kind);
            let field_doc = field.doc;
            quote! {
                #[doc = #field_doc]
                pub #name: #ty,
            }
        })
        .collect();

    let derives = if all_eq {
        quote! { #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] }
    } else {
        quote! { #[derive(Debug, Clone, Copy, PartialEq, Default)] }
    };

    quote! {
        #[doc = #doc]
        #derives
        pub struct #ident {
            #(#field_tokens)*
        }
    }
}
