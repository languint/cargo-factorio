use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::quote;

use crate::generate::ident::make_ident;
use crate::generate::types::{
    lua_any_type, map_field_type, map_parameter_type, map_return_stub, map_return_type, stub_expr,
};
use crate::schema::{Attribute, Class, Method, RuntimeApi};

/// Returns the fully-inherited attribute and method lists for `class` by walking
/// the `parent` chain. Parent members come first; child members override duplicates.
fn inherited_members<'a>(
    class: &'a Class,
    by_name: &'a HashMap<&'a str, &'a Class>,
) -> (Vec<&'a Attribute>, Vec<&'a Method>) {
    let mut attrs: Vec<&Attribute> = Vec::new();
    let mut methods: Vec<&Method> = Vec::new();

    // Collect ancestor chain (root first).
    let mut chain: Vec<&Class> = Vec::new();
    let mut current = class;
    loop {
        chain.push(current);
        match current.parent.as_deref().and_then(|p| by_name.get(p)) {
            Some(parent) => current = parent,
            None => break,
        }
    }
    chain.reverse(); // root → … → class

    let mut seen_attrs: HashSet<&str> = HashSet::new();
    let mut seen_methods: HashSet<&str> = HashSet::new();

    for ancestor in chain {
        for attr in &ancestor.attributes {
            if seen_attrs.insert(attr.name.as_str()) {
                attrs.push(attr);
            }
        }
        for method in &ancestor.methods {
            if seen_methods.insert(method.name.as_str()) {
                methods.push(method);
            }
        }
    }

    (attrs, methods)
}

fn class_names(api: &RuntimeApi) -> std::collections::BTreeSet<String> {
    api.classes.iter().map(|class| class.name.clone()).collect()
}

fn method_rust_name(name: &str) -> proc_macro2::Ident {
    if name == "type" {
        make_ident("get_type")
    } else {
        make_ident(name)
    }
}

fn generate_method(
    method: &crate::schema::Method,
    class_names: &std::collections::BTreeSet<String>,
) -> TokenStream {
    let name = method_rust_name(&method.name);
    let return_type = map_return_type(&method.return_values, class_names);
    let body = stub_expr(&map_return_stub(&method.return_values, class_names));

    let params = if method.format.takes_table {
        vec![quote!( _options: Option<crate::LuaAny> )]
    } else {
        method
            .parameters
            .iter()
            .map(|parameter| {
                let param_name = make_ident(&parameter.name);
                let param_type = map_parameter_type(parameter, class_names);
                quote!( #param_name: #param_type )
            })
            .collect::<Vec<_>>()
    };

    let doc = if method.description.is_empty() {
        None
    } else {
        Some(method.description.as_str())
    };

    if let Some(description) = doc {
        quote! {
            #[doc = #description]
            #[allow(clippy::too_many_arguments, unused_variables)]
            pub fn #name(&self, #( #params ),* ) -> #return_type #body
        }
    } else {
        quote! {
            #[allow(clippy::too_many_arguments, unused_variables)]
            pub fn #name(&self, #( #params ),* ) -> #return_type #body
        }
    }
}

/// Generates a public struct field for a Factorio API attribute (property).
///
/// Factorio attributes are Lua properties (e.g. `entity.name`), not methods, so they
/// must be struct fields, not getter methods. Writing `entity.name` in Rust then
/// correctly transpiles to `entity.name` in Lua.
fn generate_attribute(
    attribute: &crate::schema::Attribute,
    class_names: &std::collections::BTreeSet<String>,
    reserved_names: &HashSet<String>,
) -> Option<TokenStream> {
    let field_name = make_ident(&attribute.name);
    if reserved_names.contains(&field_name.to_string()) {
        return None;
    }

    let field_type = attribute
        .read_type
        .as_ref()
        .map(|api_type| map_field_type(api_type, class_names))
        .unwrap_or_else(lua_any_type);

    let doc = if attribute.description.is_empty() {
        None
    } else {
        Some(attribute.description.as_str())
    };

    if let Some(description) = doc {
        Some(quote! {
            #[doc = #description]
            pub #field_name: #field_type,
        })
    } else {
        Some(quote! {
            pub #field_name: #field_type,
        })
    }
}

fn generate_class(
    class: &Class,
    class_names: &std::collections::BTreeSet<String>,
    by_name: &HashMap<&str, &Class>,
) -> TokenStream {
    let class_name = make_ident(&class.name);

    let (all_attrs, all_methods) = inherited_members(class, by_name);

    let mut reserved_names = HashSet::new();
    for method in &all_methods {
        reserved_names.insert(method_rust_name(&method.name).to_string());
    }

    let methods = all_methods
        .iter()
        .map(|method| generate_method(method, class_names));
    let attributes = all_attrs
        .iter()
        .filter_map(|attribute| generate_attribute(attribute, class_names, &reserved_names));

    let doc = if class.description.is_empty() {
        None
    } else {
        Some(class.description.as_str())
    };
    // Attributes are public fields so field access `entity.name` transpiles to
    // `entity.name` in Lua (a property), while method calls `entity.destroy()`
    // transpile to `entity:destroy()` (a Lua method). Copy/Eq are not derived
    // because fields include String, Vec, f64, and Box<T>.
    let derive = quote!(#[derive(Debug, Clone, PartialEq, Default)]);

    if let Some(description) = doc {
        quote! {
            #[doc = #description]
            #derive
            pub struct #class_name {
                #( #attributes )*
            }

            impl #class_name {
                #( #methods )*
            }
        }
    } else {
        quote! {
            #derive
            pub struct #class_name {
                #( #attributes )*
            }

            impl #class_name {
                #( #methods )*
            }
        }
    }
}

pub fn generate_classes(api: &RuntimeApi) -> String {
    let names = class_names(api);
    let by_name: HashMap<&str, &Class> = api.classes.iter().map(|c| (c.name.as_str(), c)).collect();
    let classes = api
        .classes
        .iter()
        .map(|class| generate_class(class, &names, &by_name));
    let tokens = quote! {
        #( #classes )*
    };
    tokens.to_string()
}

pub fn generate_globals(api: &RuntimeApi) -> String {
    let names = class_names(api);
    let globals = api.global_objects.iter().map(|global| {
        let global_name = make_ident(&global.name);
        let type_name = match global.type_name.as_simple_name() {
            Some(name) => {
                let ident = make_ident(name);
                quote!(crate::classes::#ident)
            }
            None => lua_any_type(),
        };
        let _ = &names;
        // LazyLock<T> derefs to T, so `game.x` still works from user code.
        quote! {
            pub static #global_name: std::sync::LazyLock<#type_name> =
                std::sync::LazyLock::new(#type_name::default);
        }
    });

    let global_functions = api.global_functions.iter().map(|function| {
        let name = method_rust_name(&function.name);
        let return_type = map_return_type(&function.return_values, &names);
        let body = stub_expr(&map_return_stub(&function.return_values, &names));
        let params = function.parameters.iter().map(|parameter| {
            let param_name = make_ident(&parameter.name);
            let param_type = map_parameter_type(parameter, &names);
            quote!( #param_name: #param_type )
        });

        quote! {
            #[allow(unused_variables)]
            pub fn #name( #( #params ),* ) -> #return_type #body
        }
    });

    let tokens = quote! {
        #( #globals )*
        #( #global_functions )*
    };
    tokens.to_string()
}
