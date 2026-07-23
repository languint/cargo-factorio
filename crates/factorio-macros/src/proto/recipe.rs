use proc_macro::TokenStream;
use syn::{
    LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

use super::common::{
    emit_register_module, option_bool_tokens, option_f64_tokens, option_str_tokens, parse_f64_lit,
};
use super::helpers::{names_idents, proto_list_input, push_const};
use super::names::{RecipeComponent, parse_recipe_components};

pub fn recipe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as RecipesInput);
    let mut const_defs = Vec::new();
    let mut extend_items = Vec::new();

    for entry in &input.entries {
        push_const(&mut const_defs, &entry.ident, &entry.name);
        let name_lit = entry.name.as_str();
        let energy_required = option_f64_tokens(entry.energy_required);
        let category = option_str_tokens(entry.category.as_deref());
        let enabled = option_bool_tokens(entry.enabled);
        let subgroup = option_str_tokens(entry.subgroup.as_deref());
        let order = option_str_tokens(entry.order.as_deref());

        let ingredients = entry.ingredients.iter().map(|ing| {
            let name = ing.name.to_tokens();
            let amount = ing.amount;
            let fluid = ing.fluid;
            quote::quote! {
                RecipeIngredient {
                    name: #name,
                    amount: #amount,
                    fluid: #fluid,
                    ..Default::default()
                }
            }
        });
        let results = entry.results.iter().map(|prod| {
            let name = prod.name.to_tokens();
            let amount = prod.amount;
            quote::quote! {
                RecipeProduct {
                    name: #name,
                    amount: #amount,
                    ..Default::default()
                }
            }
        });

        extend_items.push(quote::quote! {
            Recipe {
                name: #name_lit,
                ingredients: &[ #( #ingredients ),* ],
                results: &[ #( #results ),* ],
                energy_required: #energy_required,
                category: #category,
                enabled: #enabled,
                subgroup: #subgroup,
                order: #order,
                ..Default::default()
            }
        });
    }

    let (names, register) = names_idents("Recipes", "register_recipes");
    TokenStream::from(emit_register_module(
        &names,
        &register,
        &const_defs,
        &extend_items,
    ))
}

proto_list_input!(
    RecipesInput,
    RecipeProtoEntry,
    "expected at least one recipe block"
);

struct RecipeProtoEntry {
    ident: syn::Ident,
    name: String,
    ingredients: Vec<RecipeComponent>,
    results: Vec<RecipeComponent>,
    energy_required: Option<f64>,
    category: Option<String>,
    enabled: Option<bool>,
    subgroup: Option<String>,
    order: Option<String>,
}

impl Parse for RecipeProtoEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let content;
        syn::braced!(content in input);

        let mut name: Option<String> = None;
        let mut ingredients: Option<Vec<RecipeComponent>> = None;
        let mut results: Option<Vec<RecipeComponent>> = None;
        let mut energy_required: Option<f64> = None;
        let mut category: Option<String> = None;
        let mut enabled: Option<bool> = None;
        let mut subgroup: Option<String> = None;
        let mut order: Option<String> = None;

        while !content.is_empty() {
            let field: syn::Ident = content.parse()?;
            let _: Token![=] = content.parse()?;
            match field.to_string().as_str() {
                "name" => {
                    let lit: LitStr = content.parse()?;
                    name = Some(lit.value());
                }
                "ingredients" => {
                    ingredients = Some(parse_recipe_components(&content)?);
                }
                "results" => {
                    results = Some(parse_recipe_components(&content)?);
                }
                "energy_required" => {
                    energy_required = Some(parse_f64_lit(&content)?);
                }
                "category" => {
                    let lit: LitStr = content.parse()?;
                    category = Some(lit.value());
                }
                "enabled" => {
                    let lit: syn::LitBool = content.parse()?;
                    enabled = Some(lit.value());
                }
                "subgroup" => {
                    let lit: LitStr = content.parse()?;
                    subgroup = Some(lit.value());
                }
                "order" => {
                    let lit: LitStr = content.parse()?;
                    order = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        field.span(),
                        format!(
                            "unknown recipe field `{other}`; expected `name`, `ingredients`, `results`, `energy_required`, `category`, `enabled`, `subgroup`, or `order`"
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
                syn::Error::new(span, "recipe block missing required field `name`")
            })?,
            ingredients: ingredients.ok_or_else(|| {
                syn::Error::new(span, "recipe block missing required field `ingredients`")
            })?,
            results: results.ok_or_else(|| {
                syn::Error::new(span, "recipe block missing required field `results`")
            })?,
            energy_required,
            category,
            enabled,
            subgroup,
            order,
        })
    }
}
