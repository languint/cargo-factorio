use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    Expr, LitStr, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

use crate::proto::common::screaming_to_const_ident;

pub fn mod_settings(input: TokenStream) -> TokenStream {
    let ModSettingsInput { prefix, groups } = parse_macro_input!(input as ModSettingsInput);

    let mut const_defs = Vec::<TokenStream2>::new();
    let mut bool_items = Vec::<TokenStream2>::new();
    let mut int_items = Vec::<TokenStream2>::new();
    let mut double_items = Vec::<TokenStream2>::new();
    let mut string_items = Vec::<TokenStream2>::new();

    for group in &groups {
        let setting_type_str = match group.stage {
            SettingStage::Startup => "startup",
            SettingStage::RuntimeGlobal => "runtime-global",
            SettingStage::RuntimePerUser => "runtime-per-user",
        };

        for entry in &group.entries {
            let const_name = screaming_to_const_ident(&entry.ident);
            let lua_name = build_lua_name(prefix.as_deref(), &entry.ident.to_string());
            let default_expr = &entry.default;

            // `pub const CASUAL_MODE: &'static str = "ms-casual-mode";`
            const_defs.push(quote::quote! {
                pub const #const_name: &'static str = #lua_name;
            });

            let lua_name_lit = lua_name.as_str();
            let item_expr = match entry.setting_type {
                SettingType::Bool => quote::quote! {
                    BoolSetting {
                        name: #lua_name_lit,
                        setting_type: #setting_type_str,
                        default_value: #default_expr,
                    }
                },
                SettingType::Int => quote::quote! {
                    IntSetting {
                        name: #lua_name_lit,
                        setting_type: #setting_type_str,
                        default_value: #default_expr,
                        minimum_value: None,
                        maximum_value: None,
                    }
                },
                SettingType::Double => quote::quote! {
                    DoubleSetting {
                        name: #lua_name_lit,
                        setting_type: #setting_type_str,
                        default_value: #default_expr,
                        minimum_value: None,
                        maximum_value: None,
                    }
                },
                SettingType::Str => quote::quote! {
                    StringSetting {
                        name: #lua_name_lit,
                        setting_type: #setting_type_str,
                        default_value: #default_expr,
                        hidden: false,
                    }
                },
            };
            // Group by setting prototype type
            match entry.setting_type {
                SettingType::Bool => bool_items.push(item_expr),
                SettingType::Int => int_items.push(item_expr),
                SettingType::Double => double_items.push(item_expr),
                SettingType::Str => string_items.push(item_expr),
            }
        }
    }

    let mut extend_calls = Vec::new();
    if !bool_items.is_empty() {
        extend_calls.push(quote::quote! { data.extend([#( #bool_items, )*]); });
    }
    if !int_items.is_empty() {
        extend_calls.push(quote::quote! { data.extend([#( #int_items, )*]); });
    }
    if !double_items.is_empty() {
        extend_calls.push(quote::quote! { data.extend([#( #double_items, )*]); });
    }
    if !string_items.is_empty() {
        extend_calls.push(quote::quote! { data.extend([#( #string_items, )*]); });
    }

    TokenStream::from(quote::quote! {
        pub struct Settings;

        impl Settings {
            #( #const_defs )*
        }

        pub fn register() {
            #( #extend_calls )*
        }
    })
}

struct ModSettingsInput {
    prefix: Option<String>,
    groups: Vec<SettingGroup>,
}

struct SettingGroup {
    stage: SettingStage,
    entries: Vec<SettingEntry>,
}

struct SettingEntry {
    ident: syn::Ident,
    setting_type: SettingType,
    default: Expr,
}

#[derive(Clone, Copy)]
enum SettingStage {
    Startup,
    RuntimeGlobal,
    RuntimePerUser,
}

#[derive(Clone, Copy)]
enum SettingType {
    Bool,
    Int,
    Double,
    Str,
}

impl Parse for ModSettingsInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut prefix: Option<String> = None;
        if input.peek(syn::Ident) {
            let fork = input.fork();
            let kw: syn::Ident = fork.parse()?;
            if kw == "prefix" && fork.peek(Token![=]) {
                let _: syn::Ident = input.parse()?;
                let _: Token![=] = input.parse()?;
                let lit: LitStr = input.parse()?;
                prefix = Some(lit.value());
                let _: Option<Token![,]> = input.parse()?;
            }
        }

        let mut groups = Vec::new();
        while !input.is_empty() {
            groups.push(input.parse::<SettingGroup>()?);
        }

        Ok(Self { prefix, groups })
    }
}

impl Parse for SettingGroup {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let stage_kw: syn::Ident = input.parse()?;
        let stage = match stage_kw.to_string().as_str() {
            "startup" => SettingStage::Startup,
            "runtime_global" => SettingStage::RuntimeGlobal,
            "runtime_per_user" => SettingStage::RuntimePerUser,
            other => {
                return Err(syn::Error::new(
                    stage_kw.span(),
                    format!(
                        "unknown setting stage `{other}`; expected `startup`, `runtime_global`, or `runtime_per_user`"
                    ),
                ));
            }
        };

        let content;
        syn::braced!(content in input);

        let mut entries = Vec::new();
        while !content.is_empty() {
            entries.push(content.parse::<SettingEntry>()?);
        }

        Ok(Self { stage, entries })
    }
}

impl Parse for SettingEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let ty: Type = input.parse()?;
        let _: Token![=] = input.parse()?;
        let default: Expr = input.parse()?;
        let _: Option<Token![,]> = input.parse()?;

        let setting_type = type_to_setting_type(&ty).ok_or_else(|| {
            syn::Error::new_spanned(
                &ty,
                "unsupported setting type; use `bool`, `i64`, `f64`, or `&'static str`",
            )
        })?;

        Ok(Self {
            ident,
            setting_type,
            default,
        })
    }
}

fn type_to_setting_type(ty: &Type) -> Option<SettingType> {
    match ty {
        Type::Path(tp) => {
            let ident = tp.path.get_ident()?.to_string();
            match ident.as_str() {
                "bool" => Some(SettingType::Bool),
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "usize" => {
                    Some(SettingType::Int)
                }
                "f32" | "f64" => Some(SettingType::Double),
                "String" => Some(SettingType::Str),
                _ => None,
            }
        }
        // &'static str or &str
        Type::Reference(tr) => {
            if let Type::Path(tp) = tr.elem.as_ref()
                && tp.path.is_ident("str")
            {
                Some(SettingType::Str)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Build the Lua setting name: `{prefix}-{kebab-case}` or just `{kebab-case}`.
fn build_lua_name(prefix: Option<&str>, snake: &str) -> String {
    let kebab = snake.replace('_', "-");
    match prefix {
        Some(p) => format!("{p}-{kebab}"),
        None => kebab,
    }
}
