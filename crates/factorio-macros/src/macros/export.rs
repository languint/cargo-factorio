use proc_macro::TokenStream;
use syn::{
    LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

pub fn export(args: TokenStream, input: TokenStream) -> TokenStream {
    let interface = if args.is_empty() {
        None
    } else {
        let parsed = parse_macro_input!(args as ExportAttributeArgs);
        match parsed.interface {
            Some(ExportInterfaceArg::Named(lit)) => Some(lit.value()),
            Some(ExportInterfaceArg::Default) | None => None,
        }
    };
    let interface_lit = interface.unwrap_or_default();
    inject_fn_or_mod_marker("__factorio_rs_export__", &interface_lit, input)
}

pub fn inline(_args: TokenStream, input: TokenStream) -> TokenStream {
    inject_fn_or_mod_marker("__factorio_rs_inline__", "", input)
}

/// Emit `__factorio_rs_export__*` / `__factorio_rs_inline__*` consts that survive expansion.
fn inject_fn_or_mod_marker(prefix: &str, value: &str, input: TokenStream) -> TokenStream {
    let Ok(item) = syn::parse::<syn::Item>(input.clone()) else {
        return input;
    };

    match item {
        syn::Item::Fn(function) => {
            let marker = syn::Ident::new(
                &format!("{prefix}{}", function.sig.ident),
                function.sig.ident.span(),
            );
            let value_lit = LitStr::new(value, proc_macro2::Span::call_site());
            TokenStream::from(quote::quote! {
                #[doc(hidden)]
                #[allow(non_upper_case_globals)]
                pub const #marker: &str = #value_lit;

                #function
            })
        }
        syn::Item::Mod(mut module) => {
            if let Some((_, items)) = &mut module.content {
                let mut marked_items = Vec::with_capacity(items.len() * 2);
                for nested in std::mem::take(items) {
                    if let syn::Item::Fn(function) = nested {
                        let marker = syn::Ident::new(
                            &format!("{prefix}{}", function.sig.ident),
                            function.sig.ident.span(),
                        );
                        let value_lit = LitStr::new(value, proc_macro2::Span::call_site());
                        marked_items.push(syn::parse_quote! {
                            #[doc(hidden)]
                            #[allow(non_upper_case_globals)]
                            pub const #marker: &str = #value_lit;
                        });
                        marked_items.push(syn::Item::Fn(function));
                    } else {
                        marked_items.push(nested);
                    }
                }
                *items = marked_items;
            }
            TokenStream::from(quote::quote! { #module })
        }
        other => TokenStream::from(quote::quote! { #other }),
    }
}

/// Parsed `#[export(...)]` interface argument (validation-only today).
#[allow(dead_code)]
enum ExportInterfaceArg {
    /// `#[export(interface)]` - remote using the mod-name default.
    Default,
    /// `#[export(interface = "name")]`.
    Named(LitStr),
}

struct ExportAttributeArgs {
    #[allow(dead_code)]
    interface: Option<ExportInterfaceArg>,
}

impl Parse for ExportAttributeArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self { interface: None });
        }
        let keyword: syn::Ident = input.parse()?;
        if keyword != "interface" {
            return Err(syn::Error::new(
                keyword.span(),
                "expected `interface` or `interface = \"...\"`",
            ));
        }
        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Ok(Self {
                interface: Some(ExportInterfaceArg::Named(input.parse()?)),
            })
        } else {
            Ok(Self {
                interface: Some(ExportInterfaceArg::Default),
            })
        }
    }
}
