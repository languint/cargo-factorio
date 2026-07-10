use quote::quote;

use crate::generate::ident::{make_ident, sanitize_ident};
use crate::schema::{Concept, RuntimeApi};

fn literal_value(api_type: &crate::schema::ApiType) -> Option<String> {
    let value = api_type.0.get("value")?;
    match value {
        serde_json::Value::String(string) => Some(string.clone()),
        serde_json::Value::Number(number) => Some(number.to_string()),
        serde_json::Value::Bool(boolean) => Some(boolean.to_string()),
        _ => None,
    }
}

fn filter_union_literals(api_type: &crate::schema::ApiType) -> Vec<String> {
    if api_type.complex_type() != Some("union") {
        return Vec::new();
    }

    api_type
        .options()
        .iter()
        .filter_map(literal_value)
        .collect()
}

fn literal_method_name(value: &str) -> String {
    sanitize_ident(value)
}

fn value_field_for_filter(filter: &str) -> Option<&'static str> {
    match filter {
        "type" => Some("type"),
        "name" => Some("name"),
        "ghost_type" => Some("type"),
        "ghost_name" => Some("name"),
        "force" => Some("force"),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FilterMethodSpec {
    pub filter: String,
    pub value_field: Option<String>,
    pub arg_count: usize,
}

fn method_name_for_spec(spec: &FilterMethodSpec) -> String {
    if spec.arg_count == 0 {
        return literal_method_name(&spec.filter);
    }
    if spec.filter == "type" {
        return "type_".to_string();
    }
    sanitize_ident(&spec.filter)
}

fn collect_filter_methods(concept: &Concept) -> Vec<FilterMethodSpec> {
    let Some(table) = concept.type_name.0.as_object() else {
        return Vec::new();
    };
    if table.get("complex_type").and_then(|value| value.as_str()) != Some("table") {
        return Vec::new();
    }

    let mut methods = Vec::new();
    let variant_groups = table
        .get("variant_parameter_groups")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let variant_group_names: std::collections::BTreeSet<String> = variant_groups
        .iter()
        .filter_map(|group| group.get("name").and_then(|value| value.as_str()))
        .map(str::to_string)
        .collect();

    if let Some(parameters) = table.get("parameters").and_then(|value| value.as_array()) {
        for parameter in parameters {
            if parameter.get("name").and_then(|value| value.as_str()) != Some("filter") {
                continue;
            }
            let Some(filter_type) = parameter
                .get("type")
                .map(|value| crate::schema::ApiType(value.clone()))
            else {
                continue;
            };
            for literal in filter_union_literals(&filter_type) {
                if value_field_for_filter(&literal).is_some() {
                    continue;
                }
                if variant_group_names.contains(&literal) {
                    continue;
                }
                methods.push(FilterMethodSpec {
                    filter: literal,
                    value_field: None,
                    arg_count: 0,
                });
            }
        }
    }

    for group in &variant_groups {
        let Some(group_name) = group.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        let arg_count = group
            .get("parameters")
            .and_then(|value| value.as_array())
            .map_or(0, |parameters| parameters.len());
        methods.push(FilterMethodSpec {
            filter: group_name.to_string(),
            value_field: value_field_for_filter(group_name).map(str::to_string),
            arg_count: arg_count.max(1),
        });
    }

    let mut seen = std::collections::BTreeSet::new();
    methods.retain(|method| seen.insert(method_name_for_spec(method)));
    methods
}

pub fn generate_event_filters(api: &RuntimeApi) -> String {
    let filter_names: std::collections::BTreeSet<String> = api
        .events
        .iter()
        .filter_map(|event| event.filter.clone())
        .collect();

    let mut all_methods = Vec::new();
    let mut filter_types = Vec::new();

    for filter_name in &filter_names {
        let Some(concept) = api
            .concepts
            .iter()
            .find(|concept| concept.name == *filter_name)
        else {
            continue;
        };

        let methods = collect_filter_methods(concept);
        all_methods.extend(methods.iter().cloned());

        let type_ident = make_ident(filter_name);
        let method_items = methods.iter().map(|method| {
            let method_ident = make_ident(&method_name_for_spec(method));
            if method.arg_count == 0 {
                quote! {
                    pub const fn #method_ident() -> EventFilterEntry { EventFilterEntry }
                }
            } else if method.arg_count == 1 {
                quote! {
                    pub const fn #method_ident(_value: &str) -> EventFilterEntry { EventFilterEntry }
                }
            } else {
                quote! {
                    pub fn #method_ident(_comparison: &str, _value: f64) -> EventFilterEntry { EventFilterEntry }
                }
            }
        });

        filter_types.push(quote! {
            #[derive(Copy, Clone, Debug, PartialEq, Eq)]
            pub struct #type_ident;

            impl #type_ident {
                #( #method_items )*
            }
        });
    }

    all_methods.sort_by(|left, right| left.filter.cmp(&right.filter));
    all_methods.dedup_by(|left, right| {
        left.filter == right.filter
            && left.value_field == right.value_field
            && left.arg_count == right.arg_count
    });

    let lookup_arms: Vec<_> = all_methods
        .iter()
        .flat_map(|method| {
            let filter = &method.filter;
            let arg_count = method.arg_count;
            let method_names: Vec<String> = if arg_count == 0 {
                vec![literal_method_name(filter)]
            } else if filter == "type" {
                vec!["type_".to_string()]
            } else {
                vec![sanitize_ident(filter)]
            };

            method_names.into_iter().filter_map(move |method_name| {
                match (&method.value_field, arg_count) {
                    (Some(field), 1) => {
                        let field = field.as_str();
                        Some(quote! {
                            #method_name => Some(FilterMethodSpec {
                                filter: #filter,
                                value_field: Some(#field),
                                arg_count: 1,
                            })
                        })
                    }
                    (None, 2) => Some(quote! {
                        #method_name => Some(FilterMethodSpec {
                            filter: #filter,
                            value_field: None,
                            arg_count: 2,
                        })
                    }),
                    (None, 0) => Some(quote! {
                        #method_name => Some(FilterMethodSpec {
                            filter: #filter,
                            value_field: None,
                            arg_count: 0,
                        })
                    }),
                    _ => None,
                }
            })
        })
        .collect();

    let types_tokens = quote! {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub struct EventFilterEntry;

        #( #filter_types )*

        pub struct FilterMethodSpec {
            pub filter: &'static str,
            pub value_field: Option<&'static str>,
            pub arg_count: usize,
        }

        pub fn filter_method_spec(method: &str) -> Option<FilterMethodSpec> {
            match method {
                #( #lookup_arms, )*
                _ => None,
            }
        }
    };

    types_tokens.to_string()
}

pub fn generate_event_data(
    api: &RuntimeApi,
    known: &crate::generate::types::KnownTypes<'_>,
) -> String {
    use crate::generate::events::event_rust_name;
    use crate::generate::types::map_copy_field_type;

    let event_data = api.events.iter().map(|event| {
        let rust_name = make_ident(&format!("{}Event", event_rust_name(&event.name)));
        let fields = event.data.iter().map(|parameter| {
            let field_name = make_ident(&parameter.name);
            let field_type = map_copy_field_type(&parameter.type_name, known);
            quote! {
                pub #field_name: #field_type,
            }
        });

        let doc = if event.description.is_empty() {
            None
        } else {
            Some(event.description.as_str())
        };

        if let Some(description) = doc {
            quote! {
                #[doc = #description]
                #[derive(Debug, Clone, Copy, PartialEq, Default)]
                pub struct #rust_name {
                    #( #fields )*
                }
            }
        } else {
            quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Default)]
                pub struct #rust_name {
                    #( #fields )*
                }
            }
        }
    });

    quote! { #( #event_data )* }.to_string()
}
