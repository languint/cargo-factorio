//! Shared bits for prototype proc macros.

use proc_macro2::TokenStream as TokenStream2;
use syn::Ident;

use super::common::screaming_to_const_ident;

macro_rules! proto_list_input {
    ($input:ident, $entry:ident, $msg:expr) => {
        struct $input {
            entries: Vec<$entry>,
        }

        impl ::syn::parse::Parse for $input {
            fn parse(input: ::syn::parse::ParseStream<'_>) -> ::syn::Result<Self> {
                let mut entries = ::std::vec::Vec::new();
                while !input.is_empty() {
                    entries.push(input.parse()?);
                }
                if entries.is_empty() {
                    return ::syn::Result::Err(input.error($msg));
                }
                ::syn::Result::Ok(Self { entries })
            }
        }
    };
}
pub(crate) use proto_list_input;

pub fn names_idents(names: &str, register: &str) -> (Ident, Ident) {
    (
        Ident::new(names, proc_macro2::Span::call_site()),
        Ident::new(register, proc_macro2::Span::call_site()),
    )
}

pub fn push_const(const_defs: &mut Vec<TokenStream2>, ident: &Ident, name: &str) {
    let const_name = screaming_to_const_ident(ident);
    const_defs.push(quote::quote! {
        pub const #const_name: &'static str = #name;
    });
}
