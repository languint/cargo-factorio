use proc_macro::TokenStream;
use syn::{
    LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

use super::common::{
    emit_register_module, option_i64_tokens, option_str_tokens, parse_f64_lit, resolve_icon_path,
};
use super::helpers::{names_idents, proto_list_input, push_const};
use super::names::{ProtoName, RecipeComponent, parse_name_list, parse_recipe_components};

pub fn technology(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TechnologiesInput);
    let mod_name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "mod".to_string());

    let mut const_defs = Vec::new();
    let mut extend_items = Vec::new();

    for entry in &input.entries {
        push_const(&mut const_defs, &entry.ident, &entry.name);
        let name_lit = entry.name.as_str();
        let icon_lit = resolve_icon_path(&entry.icon, &mod_name);
        let icon_size = option_i64_tokens(entry.icon_size);
        let order = option_str_tokens(entry.order.as_deref());
        let unit_count = entry.unit_count;
        let unit_time = entry.unit_time;

        let prerequisites = entry.prerequisites.iter().map(ProtoName::to_tokens);
        let effects = entry.unlock_recipes.iter().map(|recipe| {
            let r = recipe.to_tokens();
            quote::quote! {
                UnlockRecipeEffect {
                    recipe: #r,
                    ..Default::default()
                }
            }
        });
        let unit_ingredients = entry.unit_ingredients.iter().map(|ing| {
            let n = ing.name.to_tokens();
            let amount = ing.amount;
            quote::quote! {
                TechnologyUnitIngredient {
                    name: #n,
                    amount: #amount,
                    ..Default::default()
                }
            }
        });

        extend_items.push(quote::quote! {
            Technology {
                name: #name_lit,
                icon: #icon_lit,
                icon_size: #icon_size,
                prerequisites: &[ #( #prerequisites ),* ],
                effects: &[ #( #effects ),* ],
                unit: TechnologyUnit {
                    count: #unit_count,
                    time: #unit_time,
                    ingredients: &[ #( #unit_ingredients ),* ],
                    ..Default::default()
                },
                order: #order,
                ..Default::default()
            }
        });
    }

    let (names, register) = names_idents("Technologies", "register_technologies");
    TokenStream::from(emit_register_module(
        &names,
        &register,
        &const_defs,
        &extend_items,
    ))
}

proto_list_input!(
    TechnologiesInput,
    TechnologyProtoEntry,
    "expected at least one technology block"
);

struct TechnologyProtoEntry {
    ident: syn::Ident,
    name: String,
    icon: String,
    icon_size: Option<i64>,
    prerequisites: Vec<ProtoName>,
    unlock_recipes: Vec<ProtoName>,
    unit_count: i64,
    unit_time: f64,
    unit_ingredients: Vec<RecipeComponent>,
    order: Option<String>,
}

impl Parse for TechnologyProtoEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let content;
        syn::braced!(content in input);

        let mut name: Option<String> = None;
        let mut icon: Option<String> = None;
        let mut icon_size: Option<i64> = None;
        let mut prerequisites: Option<Vec<ProtoName>> = None;
        let mut unlock_recipes: Option<Vec<ProtoName>> = None;
        let mut unit_count: Option<i64> = None;
        let mut unit_time: Option<f64> = None;
        let mut unit_ingredients: Option<Vec<RecipeComponent>> = None;
        let mut order: Option<String> = None;

        while !content.is_empty() {
            let field: syn::Ident = content.parse()?;
            let _: Token![=] = content.parse()?;
            match field.to_string().as_str() {
                "name" => {
                    let lit: LitStr = content.parse()?;
                    name = Some(lit.value());
                }
                "icon" => {
                    let lit: LitStr = content.parse()?;
                    icon = Some(lit.value());
                }
                "icon_size" => {
                    let lit: syn::LitInt = content.parse()?;
                    icon_size = Some(lit.base10_parse()?);
                }
                "prerequisites" => {
                    prerequisites = Some(parse_name_list(&content)?);
                }
                "unlock_recipes" => {
                    unlock_recipes = Some(parse_name_list(&content)?);
                }
                "unit_count" => {
                    let lit: syn::LitInt = content.parse()?;
                    unit_count = Some(lit.base10_parse()?);
                }
                "unit_time" => {
                    unit_time = Some(parse_f64_lit(&content)?);
                }
                "unit_ingredients" => {
                    unit_ingredients = Some(parse_recipe_components(&content)?);
                }
                "order" => {
                    let lit: LitStr = content.parse()?;
                    order = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        field.span(),
                        format!(
                            "unknown technology field `{other}`; expected `name`, `icon`, `icon_size`, `prerequisites`, `unlock_recipes`, `unit_count`, `unit_time`, `unit_ingredients`, or `order`"
                        ),
                    ));
                }
            }
            let _: Option<Token![,]> = content.parse()?;
        }

        let span = ident.span();
        Ok(Self {
            ident,
            name: name.ok_or_else(|| {
                syn::Error::new(span, "technology block missing required field `name`")
            })?,
            icon: icon.ok_or_else(|| {
                syn::Error::new(span, "technology block missing required field `icon`")
            })?,
            icon_size,
            prerequisites: prerequisites.unwrap_or_default(),
            unlock_recipes: unlock_recipes.ok_or_else(|| {
                syn::Error::new(
                    span,
                    "technology block missing required field `unlock_recipes`",
                )
            })?,
            unit_count: unit_count.ok_or_else(|| {
                syn::Error::new(span, "technology block missing required field `unit_count`")
            })?,
            unit_time: unit_time.ok_or_else(|| {
                syn::Error::new(span, "technology block missing required field `unit_time`")
            })?,
            unit_ingredients: unit_ingredients.ok_or_else(|| {
                syn::Error::new(
                    span,
                    "technology block missing required field `unit_ingredients`",
                )
            })?,
            order,
        })
    }
}
