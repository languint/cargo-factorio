//! Shared prototype name / recipe-component parsing for `recipe!` and `technology!`.

use proc_macro2::TokenStream as TokenStream2;
use syn::{
    LitStr, Token,
    parse::{Parse, ParseStream},
};

/// [`LitStr`] or path (`Items::WIDGET`) used for prototype id cross-refs.
pub enum ProtoName {
    Lit(String),
    Path(syn::Path),
}

impl ProtoName {
    pub fn to_tokens(&self) -> TokenStream2 {
        match self {
            Self::Lit(s) => {
                let s = s.as_str();
                quote::quote! { #s }
            }
            Self::Path(path) => quote::quote! { #path },
        }
    }

    pub fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let expr: syn::Expr = input.parse()?;
        match expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) => Ok(Self::Lit(s.value())),
            syn::Expr::Path(path) => Ok(Self::Path(path.path)),
            other => Err(syn::Error::new_spanned(
                other,
                "expected a string literal or path (e.g. `Items::WIDGET`)",
            )),
        }
    }
}

pub struct RecipeComponent {
    pub name: ProtoName,
    pub amount: i64,
    pub fluid: bool,
}

pub fn parse_recipe_components(input: ParseStream<'_>) -> syn::Result<Vec<RecipeComponent>> {
    let content;
    syn::bracketed!(content in input);
    let mut components = Vec::new();
    while !content.is_empty() {
        components.push(content.parse::<RecipeComponent>()?);
        let _: Option<Token![,]> = content.parse()?;
    }
    Ok(components)
}

impl Parse for RecipeComponent {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        syn::braced!(content in input);

        let mut name: Option<ProtoName> = None;
        let mut amount: Option<i64> = None;
        let mut fluid = false;

        while !content.is_empty() {
            let field: syn::Ident = content.parse()?;
            let _: Token![=] = content.parse()?;
            match field.to_string().as_str() {
                "name" => {
                    name = Some(ProtoName::parse(&content)?);
                }
                "amount" => {
                    let lit: syn::LitInt = content.parse()?;
                    amount = Some(lit.base10_parse()?);
                }
                "fluid" => {
                    let lit: syn::LitBool = content.parse()?;
                    fluid = lit.value();
                }
                "type" => {
                    let lit: LitStr = content.parse()?;
                    match lit.value().as_str() {
                        "fluid" => fluid = true,
                        "item" => fluid = false,
                        other => {
                            return Err(syn::Error::new(
                                lit.span(),
                                format!(
                                    "unknown ingredient type `{other}`; expected `\"item\"` or `\"fluid\"`"
                                ),
                            ));
                        }
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        field.span(),
                        format!(
                            "unknown recipe component field `{other}`; expected `name`, `amount`, `fluid`, or `type`"
                        ),
                    ));
                }
            }
            let _: Option<Token![,]> = content.parse()?;
        }

        Ok(Self {
            name: name.ok_or_else(|| syn::Error::new(content.span(), "missing `name`"))?,
            amount: amount.ok_or_else(|| syn::Error::new(content.span(), "missing `amount`"))?,
            fluid,
        })
    }
}

pub fn parse_name_list(input: ParseStream<'_>) -> syn::Result<Vec<ProtoName>> {
    let content;
    syn::bracketed!(content in input);
    let mut items = Vec::new();
    while !content.is_empty() {
        items.push(ProtoName::parse(&content)?);
        let _: Option<Token![,]> = content.parse()?;
    }
    Ok(items)
}
