use crate::generate::ident::make_ident;
use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::ApiType;

/// Opaque placeholder for complex Factorio Lua API values.
pub fn lua_any_type() -> TokenStream {
    quote!(crate::LuaAny)
}

pub enum ReturnStub {
    Unit,
    Bool,
    Number,
    Str,
    LuaAny,
    Default,
    Option(Box<ReturnStub>),
    Vec(Box<ReturnStub>),
    Tuple(Vec<ReturnStub>),
}

pub fn return_stub_for_type(
    api_type: &ApiType,
    class_names: &std::collections::BTreeSet<String>,
) -> ReturnStub {
    if let Some(name) = api_type.as_simple_name() {
        return match name {
            "boolean" => ReturnStub::Bool,
            "string" | "LocalisedString" | "LuaLazyLoadedValueLocalisedString" => ReturnStub::Str,
            "uint" | "int" | "number" | "float" | "double" | "MapTick" | "Tick" | "uint8"
            | "uint16" | "uint32" | "uint64" | "int8" | "int16" | "int32" | "int64"
            | "ItemStackIndex" | "ItemCountType" | "InventoryIndex" => ReturnStub::Number,
            "nil" | "void" => ReturnStub::Unit,
            other if other.starts_with("defines.") => ReturnStub::Str,
            other if class_names.contains(other) => ReturnStub::Default,
            _ => ReturnStub::LuaAny,
        };
    }

    match api_type.complex_type() {
        Some("array") => ReturnStub::Vec(Box::new(
            api_type
                .child_type("value")
                .map(|value| return_stub_for_type(&value, class_names))
                .unwrap_or(ReturnStub::LuaAny),
        )),
        Some("union") => {
            let options = api_type.options();
            if options.len() == 1 {
                return_stub_for_type(&options[0], class_names)
            } else {
                ReturnStub::LuaAny
            }
        }
        Some("type") => api_type
            .child_type("value")
            .map(|value| return_stub_for_type(&value, class_names))
            .unwrap_or(ReturnStub::LuaAny),
        Some("LuaLazyLoadedValue") => api_type
            .child_type("value")
            .map(|value| return_stub_for_type(&value, class_names))
            .unwrap_or(ReturnStub::LuaAny),
        _ => ReturnStub::LuaAny,
    }
}

pub fn stub_expr(stub: &ReturnStub) -> TokenStream {
    match stub {
        ReturnStub::Unit => quote!({}),
        ReturnStub::Bool => quote!({ false }),
        ReturnStub::Number => quote!({ 0.0 }),
        ReturnStub::Str => quote!({ "" }),
        ReturnStub::LuaAny => quote!({ crate::LuaAny }),
        ReturnStub::Default => quote!({ Default::default() }),
        ReturnStub::Option(inner) => {
            let _ = inner;
            quote!({ None })
        }
        ReturnStub::Vec(inner) => {
            let _ = inner;
            quote!({ Vec::new() })
        }
        ReturnStub::Tuple(items) => {
            let values: Vec<_> = items
                .iter()
                .map(|item| match item {
                    ReturnStub::Unit => quote!(()),
                    ReturnStub::Bool => quote!(false),
                    ReturnStub::Number => quote!(0.0),
                    ReturnStub::Str => quote!(""),
                    ReturnStub::LuaAny => quote!(crate::LuaAny),
                    ReturnStub::Default => quote!(Default::default()),
                    ReturnStub::Option(_) => quote!(None),
                    ReturnStub::Vec(_) => quote!(Vec::new()),
                    ReturnStub::Tuple(_) => quote!(()),
                })
                .collect();
            quote!({ (#(#values),*) })
        }
    }
}

pub fn map_api_type(
    api_type: &ApiType,
    class_names: &std::collections::BTreeSet<String>,
) -> TokenStream {
    if let Some(name) = api_type.as_simple_name() {
        return map_simple_type(name, class_names);
    }

    match api_type.complex_type() {
        Some("array") => {
            let inner = api_type
                .child_type("value")
                .map(|value| map_api_type(&value, class_names))
                .unwrap_or_else(lua_any_type);
            quote!(Vec<#inner>)
        }
        Some("dictionary") | Some("LuaCustomTable") => lua_any_type(),
        Some("union") => {
            let options = api_type.options();
            if options.len() == 1 {
                map_api_type(&options[0], class_names)
            } else {
                lua_any_type()
            }
        }
        Some("type") => api_type
            .child_type("value")
            .map(|value| map_api_type(&value, class_names))
            .unwrap_or_else(lua_any_type),
        Some("tuple") | Some("function") | Some("literal") | Some("LuaStruct") => lua_any_type(),
        Some("LuaLazyLoadedValue") => api_type
            .child_type("value")
            .map(|value| map_api_type(&value, class_names))
            .unwrap_or_else(lua_any_type),
        Some("table") => lua_any_type(),
        _ => lua_any_type(),
    }
}

fn map_simple_type(name: &str, class_names: &std::collections::BTreeSet<String>) -> TokenStream {
    match name {
        "string" | "LocalisedString" | "LuaLazyLoadedValueLocalisedString" => quote!(&str),
        "boolean" => quote!(bool),
        "uint" | "int" | "number" | "float" | "double" | "MapTick" | "Tick" | "uint8"
        | "uint16" | "uint32" | "uint64" | "int8" | "int16" | "int32" | "int64" => {
            quote!(f64)
        }
        "nil" | "void" => quote!(()),
        "ItemStackIndex" | "ItemCountType" | "InventoryIndex" => quote!(f64),
        other if class_names.contains(other) => {
            let ident = make_ident(other);
            quote!(#ident)
        }
        other if other.starts_with("defines.") => quote!(&str),
        _ => lua_any_type(),
    }
}

/// Like [`map_api_type`] but for struct fields: uses owned `String` instead of `&str`
/// so fields don't require lifetime parameters.
pub fn map_field_type(
    api_type: &ApiType,
    class_names: &std::collections::BTreeSet<String>,
) -> TokenStream {
    if let Some(name) = api_type.as_simple_name() {
        return map_simple_field_type(name, class_names);
    }

    match api_type.complex_type() {
        Some("array") => {
            let inner = api_type
                .child_type("value")
                .map(|value| map_field_type(&value, class_names))
                .unwrap_or_else(lua_any_type);
            quote!(Vec<#inner>)
        }
        Some("dictionary") | Some("LuaCustomTable") => lua_any_type(),
        Some("union") => {
            let options = api_type.options();
            if options.len() == 1 {
                map_field_type(&options[0], class_names)
            } else {
                lua_any_type()
            }
        }
        Some("type") => api_type
            .child_type("value")
            .map(|value| map_field_type(&value, class_names))
            .unwrap_or_else(lua_any_type),
        Some("tuple") | Some("function") | Some("literal") | Some("LuaStruct") => lua_any_type(),
        Some("LuaLazyLoadedValue") => api_type
            .child_type("value")
            .map(|value| map_field_type(&value, class_names))
            .unwrap_or_else(lua_any_type),
        Some("table") => lua_any_type(),
        _ => lua_any_type(),
    }
}

fn map_simple_field_type(
    name: &str,
    class_names: &std::collections::BTreeSet<String>,
) -> TokenStream {
    match name {
        // Use owned String for struct fields — &str needs a lifetime parameter.
        "string" | "LocalisedString" | "LuaLazyLoadedValueLocalisedString" => quote!(String),
        "boolean" => quote!(bool),
        "uint" | "int" | "number" | "float" | "double" | "MapTick" | "Tick" | "uint8"
        | "uint16" | "uint32" | "uint64" | "int8" | "int16" | "int32" | "int64" => {
            quote!(f64)
        }
        "nil" | "void" => quote!(()),
        "ItemStackIndex" | "ItemCountType" | "InventoryIndex" => quote!(f64),
        other if class_names.contains(other) => {
            let ident = make_ident(other);
            // Box<T> breaks potential recursive struct cycles (e.g. LuaForce ↔ LuaTechnology).
            quote!(Box<#ident>)
        }
        other if other.starts_with("defines.") => quote!(String),
        _ => lua_any_type(),
    }
}

pub fn map_parameter_stub(
    parameter: &crate::schema::Parameter,
    class_names: &std::collections::BTreeSet<String>,
) -> ReturnStub {
    let mut stub = return_stub_for_type(&parameter.type_name, class_names);
    if parameter.optional {
        stub = ReturnStub::Option(Box::new(stub));
    }
    stub
}

pub fn map_return_stub(
    return_values: &[crate::schema::Parameter],
    class_names: &std::collections::BTreeSet<String>,
) -> ReturnStub {
    match return_values.len() {
        0 => ReturnStub::Unit,
        1 => map_parameter_stub(&return_values[0], class_names),
        count => ReturnStub::Tuple(
            return_values
                .iter()
                .take(count)
                .map(|value| map_parameter_stub(value, class_names))
                .collect(),
        ),
    }
}

pub fn map_parameter_type(
    parameter: &crate::schema::Parameter,
    class_names: &std::collections::BTreeSet<String>,
) -> TokenStream {
    let base = map_api_type(&parameter.type_name, class_names);
    if parameter.optional {
        quote!(Option<#base>)
    } else {
        base
    }
}

pub fn map_return_type(
    return_values: &[crate::schema::Parameter],
    class_names: &std::collections::BTreeSet<String>,
) -> TokenStream {
    match return_values.len() {
        0 => quote!(()),
        1 => map_parameter_type(&return_values[0], class_names),
        _ => {
            let types: Vec<_> = return_values
                .iter()
                .map(|value| map_parameter_type(value, class_names))
                .collect();
            quote!((#(#types),*))
        }
    }
}
