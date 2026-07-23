use proc_macro::TokenStream;
use syn::{
    Ident, LitInt, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

use super::common::{
    ColorLit, color_tokens, emit_register_module, option_icon_tokens, option_str_tokens,
    parse_color_lit, parse_f64_lit,
};
use super::helpers::{names_idents, proto_list_input, push_const};

pub fn tile(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TilesInput);
    let mod_name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "mod".to_string());
    let mut const_defs = Vec::new();
    let mut extend_items = Vec::new();
    for entry in &input.entries {
        push_const(&mut const_defs, &entry.ident, &entry.name);
        let name = entry.name.as_str();
        let layer = entry.layer;
        let map_color = color_tokens(entry.map_color);
        let icon = option_icon_tokens(entry.icon.as_deref(), &mod_name);
        let subgroup = option_str_tokens(entry.subgroup.as_deref());
        let order = option_str_tokens(entry.order.as_deref());
        extend_items.push(quote::quote! {
            Tile {
                name: #name,
                layer: #layer,
                map_color: #map_color,
                icon: #icon,
                subgroup: #subgroup,
                order: #order,
                ..Default::default()
            }
        });
    }
    let (names, register) = names_idents("Tiles", "register_tiles");
    TokenStream::from(emit_register_module(
        &names,
        &register,
        &const_defs,
        &extend_items,
    ))
}

proto_list_input!(TilesInput, TileEntry, "expected at least one tile block");

struct TileEntry {
    ident: Ident,
    name: String,
    layer: i64,
    map_color: ColorLit,
    icon: Option<String>,
    subgroup: Option<String>,
    order: Option<String>,
}

impl Parse for TileEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let content;
        syn::braced!(content in input);
        let mut name = None;
        let mut layer = None;
        let mut map_color = None;
        let mut red = None;
        let mut green = None;
        let mut blue = None;
        let mut alpha = None;
        let mut icon = None;
        let mut subgroup = None;
        let mut order = None;
        while !content.is_empty() {
            let field: Ident = content.parse()?;
            let _: Token![=] = content.parse()?;
            match field.to_string().as_str() {
                "name" => {
                    let lit: LitStr = content.parse()?;
                    name = Some(lit.value());
                }
                "layer" => {
                    let lit: LitInt = content.parse()?;
                    layer = Some(lit.base10_parse()?);
                }
                "map_color" => map_color = Some(parse_color_lit(&content)?),
                "r" => red = Some(parse_f64_lit(&content)?),
                "g" => green = Some(parse_f64_lit(&content)?),
                "b" => blue = Some(parse_f64_lit(&content)?),
                "a" => alpha = Some(parse_f64_lit(&content)?),
                "icon" => {
                    let lit: LitStr = content.parse()?;
                    icon = Some(lit.value());
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
                        format!("unknown tile field `{other}`"),
                    ));
                }
            }
            let _: Option<Token![,]> = content.parse()?;
        }
        let span = ident.span();
        let map_color = match map_color {
            Some(color) => color,
            None => ColorLit {
                r: red.ok_or_else(|| syn::Error::new(span, "missing `map_color` or `r`"))?,
                g: green.ok_or_else(|| syn::Error::new(span, "missing `map_color` or `g`"))?,
                b: blue.ok_or_else(|| syn::Error::new(span, "missing `map_color` or `b`"))?,
                a: alpha,
            },
        };
        Ok(Self {
            ident,
            name: name.ok_or_else(|| syn::Error::new(span, "missing `name`"))?,
            layer: layer.ok_or_else(|| syn::Error::new(span, "missing `layer`"))?,
            map_color,
            icon,
            subgroup,
            order,
        })
    }
}
