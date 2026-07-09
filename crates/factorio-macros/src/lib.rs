use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    Expr, ItemFn, Path, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
};

/// Marks a control-stage function as a Factorio event handler.
///
/// The event is inferred from the handler name and first parameter type
/// (`OnBuiltEntityEvent`). Filters are validated at compile time via a generated
/// const expression.
///
/// # Examples
///
/// Without filter:
/// ```ignore
/// #[factorio_rs::event]
/// pub fn on_singleplayer_init(event: OnSingleplayerInitEvent) {}
/// ```
///
/// With filter (filter expression is type-checked at compile time):
/// ```ignore
/// #[factorio_rs::event(filter = [OnBuiltEntityFilter::type_("inserter")])]
/// pub fn on_built_entity(event: OnBuiltEntityEvent) {}
/// ```
#[proc_macro_attribute]
pub fn event(args: TokenStream, input: TokenStream) -> TokenStream {
    let event_args = parse_macro_input!(args as EventAttributeArgs);
    let function = parse_macro_input!(input as ItemFn);

    let marker_type = match (&event_args.event, event_marker_from_param(&function)) {
        (Some(path), _) => path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        (None, Some(marker)) => Some(marker),
        (None, None) => None,
    };

    let Some(type_name) = marker_type else {
        return syn::Error::new_spanned(
            &function.sig,
            "expected an event parameter such as `event: OnBuiltEntityEvent`",
        )
        .to_compile_error()
        .into();
    };

    let Some(event_name) = lookup_event_name(&type_name) else {
        let span = event_args
            .event
            .as_ref()
            .map_or_else(|| function.sig.span(), |path| path.span());
        return syn::Error::new(span, format!("unsupported event type `{type_name}`"))
            .to_compile_error()
            .into();
    };

    if let Some(filter) = &event_args.filter {
        if lookup_event_filter_type(&type_name).is_none() {
            return syn::Error::new_spanned(
                filter,
                format!("event `{type_name}` does not support filters"),
            )
            .to_compile_error()
            .into();
        }
    }

    if function.sig.ident != event_name {
        return syn::Error::new_spanned(
            &function.sig.ident,
            format!("event handler must be named `{event_name}`"),
        )
        .to_compile_error()
        .into();
    }

    // If a filter was supplied, emit it inside a const block so rustc type-checks
    // the filter expressions (e.g. wrong method name, wrong argument type).
    let filter_check: TokenStream2 = match &event_args.filter {
        Some(filter_expr) => quote::quote! {
            const _: () = { let _ = #filter_expr; };
        },
        None => TokenStream2::new(),
    };

    TokenStream::from(quote::quote! {
        #[allow(dead_code)]
        #function

        #filter_check
    })
}

struct EventAttributeArgs {
    event: Option<Path>,
    filter: Option<Expr>,
}

impl Parse for EventAttributeArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                event: None,
                filter: None,
            });
        }

        // `filter = [...]` without an explicit event type
        if input.peek(syn::Ident) && input.peek2(Token![=]) {
            let keyword: syn::Ident = input.parse()?;
            if keyword != "filter" {
                return Err(syn::Error::new(
                    keyword.span(),
                    "expected `filter` or an event type such as `OnBuiltEntity`",
                ));
            }
            input.parse::<Token![=]>()?;
            return Ok(Self {
                event: None,
                filter: Some(input.parse::<Expr>()?),
            });
        }

        let event: Path = input.parse()?;
        let filter = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let keyword: syn::Ident = input.parse()?;
            if keyword != "filter" {
                return Err(syn::Error::new(
                    keyword.span(),
                    "expected `filter` after event type",
                ));
            }
            input.parse::<Token![=]>()?;
            Some(input.parse::<Expr>()?)
        } else {
            None
        };

        Ok(Self {
            event: Some(event),
            filter,
        })
    }
}

fn event_marker_from_param(function: &ItemFn) -> Option<String> {
    let syn::FnArg::Typed(pat_type) = function.sig.inputs.first()? else {
        return None;
    };
    event_marker_from_type(&pat_type.ty)
}

fn event_marker_from_type(ty: &Type) -> Option<String> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    let segments = &type_path.path.segments;
    if segments.len() == 1 {
        let ident = segments[0].ident.to_string();
        return ident.strip_suffix("Event").map(str::to_string);
    }
    None
}

fn lookup_event_name(type_name: &str) -> Option<&'static str> {
    include!(concat!(env!("OUT_DIR"), "/event_lookup.rs"))
}

fn lookup_event_filter_type(type_name: &str) -> Option<&'static str> {
    include!(concat!(env!("OUT_DIR"), "/event_filter_lookup.rs"))
}

/// Marks a file or inline `mod` as control-stage code for transpilation.
#[proc_macro_attribute]
pub fn control(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

/// Marks a file or inline `mod` as shared-stage code for transpilation.
#[proc_macro_attribute]
pub fn shared(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

/// Marks a file or inline `mod` as data-stage code for transpilation.
#[proc_macro_attribute]
pub fn data(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

/// Declares a control-stage module from a block of items.
#[proc_macro]
pub fn control_mod(input: TokenStream) -> TokenStream {
    wrap_stage_module("control", input)
}

/// Declares a shared-stage module from a block of items.
#[proc_macro]
pub fn shared_mod(input: TokenStream) -> TokenStream {
    wrap_stage_module("shared", input)
}

/// Declares a data-stage module from a block of items.
#[proc_macro]
pub fn data_mod(input: TokenStream) -> TokenStream {
    wrap_stage_module("data", input)
}

fn wrap_stage_module(stage: &str, input: TokenStream) -> TokenStream {
    let module_name = syn::Ident::new(
        &format!("__factorio_{stage}"),
        proc_macro2::Span::call_site(),
    );
    let items = proc_macro2::TokenStream::from(input);
    TokenStream::from(quote::quote! {
        #[doc(hidden)]
        mod #module_name { #items }
    })
}
