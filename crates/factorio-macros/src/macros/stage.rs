use proc_macro::TokenStream;
use syn::LitStr;

pub fn settings(_args: TokenStream, input: TokenStream) -> TokenStream {
    inject_stage_marker("settings", input)
}

pub fn settings_updates(_args: TokenStream, input: TokenStream) -> TokenStream {
    inject_stage_marker("settings_updates", input)
}

pub fn settings_final_fixes(_args: TokenStream, input: TokenStream) -> TokenStream {
    inject_stage_marker("settings_final_fixes", input)
}

pub fn data(_args: TokenStream, input: TokenStream) -> TokenStream {
    inject_stage_marker("data", input)
}

pub fn data_updates(_args: TokenStream, input: TokenStream) -> TokenStream {
    inject_stage_marker("data_updates", input)
}

pub fn data_final_fixes(_args: TokenStream, input: TokenStream) -> TokenStream {
    inject_stage_marker("data_final_fixes", input)
}

pub fn control(_args: TokenStream, input: TokenStream) -> TokenStream {
    inject_stage_marker("control", input)
}

pub fn shared(_args: TokenStream, input: TokenStream) -> TokenStream {
    inject_stage_marker("shared", input)
}

/// Emit a durable `__factorio_rs_stage` marker that survives `-Zunpretty=expanded`.
fn inject_stage_marker(stage: &str, input: TokenStream) -> TokenStream {
    let stage_lit = LitStr::new(stage, proc_macro2::Span::call_site());
    let marker: syn::Item = syn::parse_quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        const __factorio_rs_stage: &str = #stage_lit;
    };

    if let Ok(mut item) = syn::parse::<syn::Item>(input.clone()) {
        if let syn::Item::Mod(module) = &mut item
            && let Some((_, items)) = &mut module.content
        {
            items.insert(0, marker);
            return TokenStream::from(quote::quote! { #item });
        }
        return TokenStream::from(quote::quote! {
            #marker
            #item
        });
    }

    let body = proc_macro2::TokenStream::from(input);
    TokenStream::from(quote::quote! {
        #marker
        #body
    })
}

pub fn settings_mod(input: TokenStream) -> TokenStream {
    wrap_stage_module("settings", input)
}

pub fn settings_updates_mod(input: TokenStream) -> TokenStream {
    wrap_stage_module("settings_updates", input)
}

pub fn settings_final_fixes_mod(input: TokenStream) -> TokenStream {
    wrap_stage_module("settings_final_fixes", input)
}

pub fn data_mod(input: TokenStream) -> TokenStream {
    wrap_stage_module("data", input)
}

pub fn data_updates_mod(input: TokenStream) -> TokenStream {
    wrap_stage_module("data_updates", input)
}

pub fn data_final_fixes_mod(input: TokenStream) -> TokenStream {
    wrap_stage_module("data_final_fixes", input)
}

pub fn control_mod(input: TokenStream) -> TokenStream {
    wrap_stage_module("control", input)
}

pub fn shared_mod(input: TokenStream) -> TokenStream {
    wrap_stage_module("shared", input)
}

fn wrap_stage_module(stage: &str, input: TokenStream) -> TokenStream {
    let module_name = syn::Ident::new(
        &format!("__factorio_{stage}"),
        proc_macro2::Span::call_site(),
    );
    let stage_lit = LitStr::new(stage, proc_macro2::Span::call_site());
    let items = proc_macro2::TokenStream::from(input);
    // Inject `__factorio_rs_stage` so `-Zunpretty=expanded` discovery can recover
    // the stage (the wrapper name `__factorio_{stage}` is not a path-based module).
    TokenStream::from(quote::quote! {
        #[doc(hidden)]
        mod #module_name {
            #[doc(hidden)]
            #[allow(non_upper_case_globals)]
            const __factorio_rs_stage: &str = #stage_lit;

            #items
        }
    })
}
