use crate::generate::ident::make_ident;
use convert_case::{Case, Casing};
use quote::quote;

use crate::schema::RuntimeApi;

pub struct EventMapping {
    pub rust_name: String,
    pub lua_name: String,
    pub filter: Option<String>,
}

pub fn event_rust_name(lua_name: &str) -> String {
    if lua_name.contains('_') {
        return lua_name.to_case(Case::Pascal);
    }

    if lua_name
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
    {
        return lua_name.to_string();
    }

    lua_name.to_case(Case::Pascal)
}

pub fn collect_event_mappings(api: &RuntimeApi) -> Vec<EventMapping> {
    let mut mappings: Vec<EventMapping> = api
        .events
        .iter()
        .map(|event| EventMapping {
            rust_name: event_rust_name(&event.name),
            lua_name: event.name.clone(),
            filter: event.filter.clone(),
        })
        .collect();

    mappings.sort_by(|left, right| left.rust_name.cmp(&right.rust_name));
    mappings.dedup_by(|left, right| left.rust_name == right.rust_name);
    mappings
}

pub fn generate_events(api: &RuntimeApi) -> String {
    let mappings = collect_event_mappings(api);
    let event_types = mappings.iter().map(|mapping| {
        let rust_name = make_ident(&mapping.rust_name);
        let lua_name = &mapping.lua_name;
        let event_data_alias = make_ident(&format!("{}Event", mapping.rust_name));
        let doc = api
            .events
            .iter()
            .find(|event| event.name == *lua_name)
            .map(|event| event.description.as_str())
            .filter(|description| !description.is_empty());

        let event_data_reexport = quote! {
            pub use crate::event_data::#event_data_alias;
        };

        let filter_type_reexport = mapping.filter.as_ref().map(|filter| {
            let filter_alias = make_ident(&format!("{}Filter", mapping.rust_name));
            let filter_ident = make_ident(filter);
            quote! {
                pub use crate::event_filters::#filter_ident as #filter_alias;
            }
        });

        if let Some(description) = doc {
            quote! {
                #[doc = #description]
                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                pub struct #rust_name;

                impl #rust_name {
                    pub const NAME: &'static str = #lua_name;
                }

                #event_data_reexport
                #filter_type_reexport
            }
        } else {
            quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                pub struct #rust_name;

                impl #rust_name {
                    pub const NAME: &'static str = #lua_name;
                }

                #event_data_reexport
                #filter_type_reexport
            }
        }
    });

    let tokens = quote! {
        #( #event_types )*
    };

    tokens.to_string()
}

pub fn generate_event_map(mappings: &[EventMapping]) -> String {
    let name_arms = mappings.iter().map(|mapping| {
        let rust_name = &mapping.rust_name;
        let lua_name = &mapping.lua_name;
        quote! {
            #rust_name => Some(#lua_name)
        }
    });

    let filter_arms = mappings.iter().filter_map(|mapping| {
        let rust_name = &mapping.rust_name;
        let filter = mapping.filter.as_ref()?;
        Some(quote! {
            #rust_name => Some(#filter)
        })
    });

    let tokens = quote! {
        pub fn event_type_to_name(type_name: &str) -> Option<&'static str> {
            match type_name {
                #( #name_arms, )*
                _ => None,
            }
        }

        pub fn event_filter_type(type_name: &str) -> Option<&'static str> {
            match type_name {
                #( #filter_arms, )*
                _ => None,
            }
        }
    };

    tokens.to_string()
}

pub fn generate_event_lookup(mappings: &[EventMapping]) -> String {
    let arms = mappings.iter().map(|mapping| {
        let rust_name = &mapping.rust_name;
        let lua_name = &mapping.lua_name;
        quote! {
            #rust_name => Some(#lua_name)
        }
    });

    let tokens = quote! {
        match type_name {
            #( #arms, )*
            _ => None,
        }
    };

    tokens.to_string()
}

pub fn generate_event_module_lookup(mappings: &[EventMapping]) -> String {
    let arms = mappings.iter().map(|mapping| {
        let module_name = mapping.lua_name.to_case(Case::Snake);
        let rust_name = &mapping.rust_name;
        quote! {
            #module_name => Some(#rust_name)
        }
    });

    let tokens = quote! {
        match module_name {
            #( #arms, )*
            _ => None,
        }
    };

    tokens.to_string()
}

pub fn generate_event_filter_lookup(mappings: &[EventMapping]) -> String {
    let arms = mappings.iter().filter_map(|mapping| {
        let rust_name = &mapping.rust_name;
        let filter = mapping.filter.as_ref()?;
        Some(quote! {
            #rust_name => Some(#filter)
        })
    });

    let tokens = quote! {
        match type_name {
            #( #arms, )*
            _ => None,
        }
    };

    tokens.to_string()
}
