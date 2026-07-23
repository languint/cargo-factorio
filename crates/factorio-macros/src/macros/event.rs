use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    Expr, ItemFn, LitStr, Path, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
};

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
            .map_or_else(|| function.sig.span(), Spanned::span);
        return syn::Error::new(span, format!("unsupported event type `{type_name}`"))
            .to_compile_error()
            .into();
    };

    if let Some(filter) = &event_args.filter
        && lookup_event_filter_type(&type_name).is_none()
    {
        return syn::Error::new_spanned(
            filter,
            format!("event `{type_name}` does not support filters"),
        )
        .to_compile_error()
        .into();
    }

    if function.sig.ident != event_name {
        return syn::Error::new_spanned(
            &function.sig.ident,
            format!("event handler must be named `{event_name}`"),
        )
        .to_compile_error()
        .into();
    }

    let filter_check: TokenStream2 =
        event_args
            .filter
            .as_ref()
            .map_or_else(TokenStream2::new, |filter_expr| {
                quote::quote! {
                    const _: () = { let _ = #filter_expr; };
                }
            });

    let event_marker = syn::Ident::new(
        &format!("__factorio_rs_event__{}", function.sig.ident),
        function.sig.ident.span(),
    );
    let filter_lit = event_args.filter.as_ref().map_or_else(
        || LitStr::new("", proc_macro2::Span::call_site()),
        |expr| {
            let tokens = quote::quote! { #expr };
            LitStr::new(&tokens.to_string(), proc_macro2::Span::call_site())
        },
    );

    TokenStream::from(quote::quote! {
        #[allow(dead_code)]
        #function

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        pub const #event_marker: &str = #filter_lit;

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
