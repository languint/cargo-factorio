use crate::generate::ident::make_ident;
use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::Define;

fn define_value_ident(name: &str) -> proc_macro2::Ident {
    make_ident(&name.to_ascii_uppercase())
}

fn generate_define_module(define: &Define, path: Vec<&str>) -> TokenStream {
    let module_name = make_ident(&define.name);
    let doc = if define.description.is_empty() {
        None
    } else {
        Some(define.description.as_str())
    };

    let value_items = define.values.iter().map(|value| {
        let ident = define_value_ident(&value.name);
        let name = &value.name;
        if value.description.is_empty() {
            quote! {
                pub const #ident: &'static str = #name;
            }
        } else {
            let description = &value.description;
            quote! {
                #[doc = #description]
                pub const #ident: &'static str = #name;
            }
        }
    });

    let subkey_modules = define.subkeys.iter().map(|subkey| {
        let mut subpath = path.clone();
        subpath.push(&define.name);
        generate_define_module(subkey, subpath)
    });

    if let Some(description) = doc {
        quote! {
            #[doc = #description]
            pub mod #module_name {
                #( #value_items )*
                #( #subkey_modules )*
            }
        }
    } else {
        quote! {
            pub mod #module_name {
                #( #value_items )*
                #( #subkey_modules )*
            }
        }
    }
}

pub fn generate_defines(defines: &[Define]) -> String {
    let modules = defines
        .iter()
        .map(|define| generate_define_module(define, Vec::new()));
    let tokens = quote! {
        #( #modules )*
    };
    tokens.to_string()
}
