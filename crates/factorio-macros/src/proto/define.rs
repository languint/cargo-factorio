//! Declarative `define_proto!` DSL for data-stage prototype macros.
//!
//! Generates the entry struct, `Parse` impl, list input wrapper, and expand fn.

/// Define a prototype proc-macro implementation.
///
/// See the module docs and call sites for field kinds and `emit:` forms.
macro_rules! define_proto {
    (
        fn: $fn_name:ident,
        ty: $rust_ty:ident,
        names: $names:literal,
        register: $register:literal,
        fields: {
            $($fields:tt)*
        }
        $(, emit: { $($emit:tt)* })?
    ) => {
        $crate::proto::define::define_proto! {
            @entry
            fn: $fn_name,
            ty: $rust_ty,
            names: $names,
            register: $register,
            fields: { $($fields)* },
            emit: { $($($emit)*)? }
        }
    };

    (
        @entry
        fn: $fn_name:ident,
        ty: $rust_ty:ident,
        names: $names:literal,
        register: $register:literal,
        fields: { $($fields:tt)* },
        emit: { $($emit:tt)* }
    ) => {
        $crate::proto::define::define_proto! {
            @parse_fields
            fn: $fn_name,
            ty: $rust_ty,
            names: $names,
            register: $register,
            acc: [],
            rest: { $($fields)* },
            emit: { $($emit)* }
        }
    };

    (
        @parse_fields
        fn: $fn_name:ident,
        ty: $rust_ty:ident,
        names: $names:literal,
        register: $register:literal,
        acc: [$($acc:tt)*],
        rest: {
            $field:ident : $kind:ident = $default:expr
            $(, $($rest:tt)*)?
        },
        emit: { $($emit:tt)* }
    ) => {
        $crate::proto::define::define_proto! {
            @parse_fields
            fn: $fn_name,
            ty: $rust_ty,
            names: $names,
            register: $register,
            acc: [$($acc)* ($field, $kind, default($default),)],
            rest: { $($($rest)*)? },
            emit: { $($emit)* }
        }
    };

    (
        @parse_fields
        fn: $fn_name:ident,
        ty: $rust_ty:ident,
        names: $names:literal,
        register: $register:literal,
        acc: [$($acc:tt)*],
        rest: {
            $field:ident : $kind:ident
            $(, $($rest:tt)*)?
        },
        emit: { $($emit:tt)* }
    ) => {
        $crate::proto::define::define_proto! {
            @parse_fields
            fn: $fn_name,
            ty: $rust_ty,
            names: $names,
            register: $register,
            acc: [$($acc)* ($field, $kind, required,)],
            rest: { $($($rest)*)? },
            emit: { $($emit)* }
        }
    };

    (
        @parse_fields
        fn: $fn_name:ident,
        ty: $rust_ty:ident,
        names: $names:literal,
        register: $register:literal,
        acc: [$($acc:tt)*],
        rest: {},
        emit: { $($emit:tt)* }
    ) => {
        $crate::proto::define::define_proto! {
            @generate
            fn: $fn_name,
            ty: $rust_ty,
            names: $names,
            register: $register,
            fields: [$($acc)*],
            emit: { $($emit)* }
        }
    };

    (
        @generate
        fn: $fn_name:ident,
        ty: $rust_ty:ident,
        names: $names:literal,
        register: $register:literal,
        fields: [$(
            ($field:ident, $kind:ident, $mode:ident $(($default:expr))?,)
        )*],
        emit: { $($emit:tt)* }
    ) => {
        $crate::proto::define::define_proto! {
            @struct_and_parse
            error: stringify!($fn_name),
            fields: [$(($field, $kind, $mode $( ($default) )?,))*],
        }

        $crate::proto::helpers::proto_list_input!(
            __ProtoInput,
            __ProtoEntry,
            concat!("expected at least one ", stringify!($fn_name), " block")
        );

        pub fn $fn_name(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            use ::syn::parse_macro_input;
            use $crate::proto::common::emit_register_module;
            use $crate::proto::helpers::{names_idents, push_const};

            let input = parse_macro_input!(input as __ProtoInput);
            let needs_mod = $crate::proto::define::define_proto!(@needs_mod_name [$($kind)*]);
            #[allow(unused_variables)]
            let mod_name = if needs_mod {
                ::std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "mod".to_string())
            } else {
                ::std::string::String::new()
            };

            let mut const_defs = ::std::vec::Vec::new();
            let mut extend_items = ::std::vec::Vec::new();
            for entry in &input.entries {
                push_const(&mut const_defs, &entry.ident, &entry.name);
                let item = $crate::proto::define::define_proto! {
                    @emit_one
                    ty: $rust_ty,
                    entry: entry,
                    mod_name: &mod_name,
                    fields: [$(($field, $kind),)*],
                    emit: { $($emit)* }
                };
                extend_items.push(item);
            }
            let (names, register) = names_idents($names, $register);
            ::proc_macro::TokenStream::from(emit_register_module(
                &names,
                &register,
                &const_defs,
                &extend_items,
            ))
        }
    };

    (
        @struct_and_parse
        error: $error:expr,
        fields: [$(
            ($field:ident, $kind:ident, $mode:ident $(($default:expr))?,)
        )*],
    ) => {
        struct __ProtoEntry {
            ident: ::syn::Ident,
            $(
                $field: $crate::proto::define::define_proto!(@storage_ty $kind),
            )*
        }

        impl ::syn::parse::Parse for __ProtoEntry {
            #[allow(clippy::too_many_lines)]
            fn parse(input: ::syn::parse::ParseStream<'_>) -> ::syn::Result<Self> {
                #[allow(unused_imports)]
                use ::syn::{Ident, LitBool, LitInt, LitStr, Token};
                #[allow(unused_imports)]
                use $crate::proto::common::{parse_color_lit, parse_f64_lit, parse_str_list};

                let ident: Ident = input.parse()?;
                let content;
                ::syn::braced!(content in input);

                $(
                    #[allow(unused_mut, unused_assignments)]
                    let mut $field: ::std::option::Option<
                        $crate::proto::define::define_proto!(@option_inner_ty $kind)
                    > = None;
                )*

                while !content.is_empty() {
                    let field: Ident = content.parse()?;
                    let _: Token![=] = content.parse()?;
                    match field.to_string().as_str() {
                        $(
                            stringify!($field) => {
                                $crate::proto::define::define_proto! {
                                    @parse_arm
                                    field: $field,
                                    kind: $kind,
                                    content: content,
                                }
                            }
                        )*
                        other => {
                            return ::syn::Result::Err(::syn::Error::new(
                                field.span(),
                                ::std::format!("unknown {} field `{other}`", $error),
                            ));
                        }
                    }
                    let _: ::std::option::Option<Token![,]> = content.parse()?;
                }

                let span = ident.span();
                ::syn::Result::Ok(Self {
                    ident,
                    $(
                        $field: $crate::proto::define::define_proto! {
                            @finish_field
                            field: $field,
                            kind: $kind,
                            mode: $mode,
                            span: span,
                            $(default: $default,)?
                        },
                    )*
                })
            }
        }
    };

    (@storage_ty str) => { ::std::string::String };
    (@storage_ty opt_str) => { ::std::option::Option<::std::string::String> };
    (@storage_ty i64) => { i64 };
    (@storage_ty opt_i64) => { ::std::option::Option<i64> };
    (@storage_ty f64) => { f64 };
    (@storage_ty opt_f64) => { ::std::option::Option<f64> };
    (@storage_ty bool) => { bool };
    (@storage_ty opt_bool) => { ::std::option::Option<bool> };
    (@storage_ty str_list) => { ::std::vec::Vec<::std::string::String> };
    (@storage_ty opt_flags) => { ::std::option::Option<::std::vec::Vec<::std::string::String>> };
    (@storage_ty opt_icon) => { ::std::option::Option<::std::string::String> };
    (@storage_ty req_icon) => { ::std::string::String };
    (@storage_ty color) => { $crate::proto::common::ColorLit };
    (@storage_ty parse_str) => { ::std::string::String };
    (@storage_ty parse_i64) => { ::std::option::Option<i64> };
    (@storage_ty parse_f64) => { ::std::option::Option<f64> };

    (@option_inner_ty str) => { ::std::string::String };
    (@option_inner_ty opt_str) => { ::std::string::String };
    (@option_inner_ty i64) => { i64 };
    (@option_inner_ty opt_i64) => { i64 };
    (@option_inner_ty f64) => { f64 };
    (@option_inner_ty opt_f64) => { f64 };
    (@option_inner_ty bool) => { bool };
    (@option_inner_ty opt_bool) => { bool };
    (@option_inner_ty str_list) => { ::std::vec::Vec<::std::string::String> };
    (@option_inner_ty opt_flags) => { ::std::vec::Vec<::std::string::String> };
    (@option_inner_ty opt_icon) => { ::std::string::String };
    (@option_inner_ty req_icon) => { ::std::string::String };
    (@option_inner_ty color) => { $crate::proto::common::ColorLit };
    (@option_inner_ty parse_str) => { ::std::string::String };
    (@option_inner_ty parse_i64) => { i64 };
    (@option_inner_ty parse_f64) => { f64 };

    (@parse_arm field: $field:ident, kind: str, content: $content:ident,) => {{
        let lit: LitStr = $content.parse()?;
        $field = Some(lit.value());
    }};
    (@parse_arm field: $field:ident, kind: opt_str, content: $content:ident,) => {{
        let lit: LitStr = $content.parse()?;
        $field = Some(lit.value());
    }};
    (@parse_arm field: $field:ident, kind: opt_icon, content: $content:ident,) => {{
        let lit: LitStr = $content.parse()?;
        $field = Some(lit.value());
    }};
    (@parse_arm field: $field:ident, kind: req_icon, content: $content:ident,) => {{
        let lit: LitStr = $content.parse()?;
        $field = Some(lit.value());
    }};
    (@parse_arm field: $field:ident, kind: parse_str, content: $content:ident,) => {{
        let lit: LitStr = $content.parse()?;
        $field = Some(lit.value());
    }};
    (@parse_arm field: $field:ident, kind: i64, content: $content:ident,) => {{
        let lit: LitInt = $content.parse()?;
        $field = Some(lit.base10_parse()?);
    }};
    (@parse_arm field: $field:ident, kind: opt_i64, content: $content:ident,) => {{
        let lit: LitInt = $content.parse()?;
        $field = Some(lit.base10_parse()?);
    }};
    (@parse_arm field: $field:ident, kind: parse_i64, content: $content:ident,) => {{
        let lit: LitInt = $content.parse()?;
        $field = Some(lit.base10_parse()?);
    }};
    (@parse_arm field: $field:ident, kind: f64, content: $content:ident,) => {{
        $field = Some(parse_f64_lit(&$content)?);
    }};
    (@parse_arm field: $field:ident, kind: opt_f64, content: $content:ident,) => {{
        $field = Some(parse_f64_lit(&$content)?);
    }};
    (@parse_arm field: $field:ident, kind: parse_f64, content: $content:ident,) => {{
        $field = Some(parse_f64_lit(&$content)?);
    }};
    (@parse_arm field: $field:ident, kind: bool, content: $content:ident,) => {{
        let lit: LitBool = $content.parse()?;
        $field = Some(lit.value());
    }};
    (@parse_arm field: $field:ident, kind: opt_bool, content: $content:ident,) => {{
        let lit: LitBool = $content.parse()?;
        $field = Some(lit.value());
    }};
    (@parse_arm field: $field:ident, kind: str_list, content: $content:ident,) => {{
        $field = Some(parse_str_list(&$content)?);
    }};
    (@parse_arm field: $field:ident, kind: opt_flags, content: $content:ident,) => {{
        $field = Some(parse_str_list(&$content)?);
    }};
    (@parse_arm field: $field:ident, kind: color, content: $content:ident,) => {{
        $field = Some(parse_color_lit(&$content)?);
    }};

    (
        @finish_field
        field: $field:ident,
        kind: $kind:ident,
        mode: required,
        span: $span:ident,
    ) => {
        $crate::proto::define::define_proto! {
            @finish_required
            field: $field,
            kind: $kind,
            span: $span,
        }
    };

    (
        @finish_field
        field: $field:ident,
        kind: $kind:ident,
        mode: default,
        span: $span:ident,
        default: $default:expr,
    ) => {
        $field.unwrap_or_else(|| ::std::convert::Into::into($default))
    };

    (@finish_required field: $field:ident, kind: opt_str, span: $span:ident,) => { $field };
    (@finish_required field: $field:ident, kind: opt_i64, span: $span:ident,) => { $field };
    (@finish_required field: $field:ident, kind: opt_f64, span: $span:ident,) => { $field };
    (@finish_required field: $field:ident, kind: opt_bool, span: $span:ident,) => { $field };
    (@finish_required field: $field:ident, kind: opt_flags, span: $span:ident,) => { $field };
    (@finish_required field: $field:ident, kind: opt_icon, span: $span:ident,) => { $field };
    (@finish_required field: $field:ident, kind: parse_i64, span: $span:ident,) => { $field };
    (@finish_required field: $field:ident, kind: parse_f64, span: $span:ident,) => { $field };
    (@finish_required field: $field:ident, kind: $kind:ident, span: $span:ident,) => {
        $field.ok_or_else(|| {
            ::syn::Error::new($span, ::std::format!("missing `{}`", stringify!($field)))
        })?
    };

    (@needs_mod_name []) => { false };
    (@needs_mod_name [opt_icon $($rest:ident)*]) => { true };
    (@needs_mod_name [req_icon $($rest:ident)*]) => { true };
    (@needs_mod_name [$kind:ident $($rest:ident)*]) => {
        $crate::proto::define::define_proto!(@needs_mod_name [$($rest)*])
    };

    // Custom emit list: bind all declared fields, then overlay energy/none forms.
    (
        @emit_one
        ty: $rust_ty:ident,
        entry: $entry:ident,
        mod_name: $mod_name:expr,
        fields: [$(($field:ident, $kind:ident),)*],
        emit: {
            $($emit_field:ident $(: $emit_kind:ident $(($($emit_args:tt)*))?)?),* $(,)?
        }
    ) => {{
        $(
            #[allow(unused_variables)]
            let $field = $crate::proto::define::define_proto! {
                @tokens_of
                entry: $entry,
                mod_name: $mod_name,
                field: $field,
                kind: $kind,
            };
        )*
        $(
            $crate::proto::define::define_proto! {
                @bind_emit_special
                entry: $entry,
                emit_field: $emit_field,
                $($emit_kind $( ( $($emit_args)* ) )?,)?
            }
        )*
        ::quote::quote! {
            $rust_ty {
                $(
                    $emit_field: #$emit_field,
                )*
                ..Default::default()
            }
        }
    }};

    // Default emit: all non-parse_* fields
    (
        @emit_one
        ty: $rust_ty:ident,
        entry: $entry:ident,
        mod_name: $mod_name:expr,
        fields: [$(($field:ident, $kind:ident),)*],
        emit: {}
    ) => {{
        use ::proc_macro2::TokenStream as TokenStream2;
        $(
            $crate::proto::define::define_proto! {
                @bind_default_emit
                entry: $entry,
                mod_name: $mod_name,
                field: $field,
                kind: $kind,
            }
        )*
        let mut field_tokens = TokenStream2::new();
        $(
            $crate::proto::define::define_proto! {
                @push_emit_field
                tokens: field_tokens,
                field: $field,
                kind: $kind,
            }
        )*
        ::quote::quote! {
            $rust_ty {
                #field_tokens
                ..Default::default()
            }
        }
    }};

    (@push_emit_field tokens: $tokens:ident, field: $field:ident, kind: parse_str,) => {};
    (@push_emit_field tokens: $tokens:ident, field: $field:ident, kind: parse_i64,) => {};
    (@push_emit_field tokens: $tokens:ident, field: $field:ident, kind: parse_f64,) => {};
    (@push_emit_field tokens: $tokens:ident, field: $field:ident, kind: $kind:ident,) => {
        {
            let value = &$field;
            $tokens.extend(::quote::quote! { $field: #value, });
        }
    };

    (@bind_default_emit entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: parse_str,) => {};
    (@bind_default_emit entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: parse_i64,) => {};
    (@bind_default_emit entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: parse_f64,) => {};
    (@bind_default_emit entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: $kind:ident,) => {
        let $field = $crate::proto::define::define_proto! {
            @tokens_of
            entry: $entry,
            mod_name: $mod_name,
            field: $field,
            kind: $kind,
        };
    };

    (
        @bind_emit_special
        entry: $entry:ident,
        emit_field: $emit_field:ident,
    ) => {};

    (
        @bind_emit_special
        entry: $entry:ident,
        emit_field: $emit_field:ident,
        energy ($type_field:ident, $priority_field:ident),
    ) => {
        let $emit_field = $crate::proto::common::energy_source_tokens(
            &$entry.$type_field,
            $entry.$priority_field.as_deref(),
        );
    };

    (
        @bind_emit_special
        entry: $entry:ident,
        emit_field: $emit_field:ident,
        none,
    ) => {
        let $emit_field = ::quote::quote! { None };
    };

    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: str,) => {{
        let v = $entry.$field.as_str();
        ::quote::quote! { #v }
    }};
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: parse_str,) => {{
        let v = $entry.$field.as_str();
        ::quote::quote! { #v }
    }};
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: opt_str,) => {
        $crate::proto::common::option_str_tokens($entry.$field.as_deref())
    };
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: i64,) => {{
        let v = $entry.$field;
        ::quote::quote! { #v }
    }};
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: opt_i64,) => {
        $crate::proto::common::option_i64_tokens($entry.$field)
    };
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: parse_i64,) => {
        $crate::proto::common::option_i64_tokens($entry.$field)
    };
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: f64,) => {{
        let v = $entry.$field;
        ::quote::quote! { #v }
    }};
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: opt_f64,) => {
        $crate::proto::common::option_f64_tokens($entry.$field)
    };
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: parse_f64,) => {
        $crate::proto::common::option_f64_tokens($entry.$field)
    };
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: bool,) => {{
        let v = $entry.$field;
        ::quote::quote! { #v }
    }};
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: opt_bool,) => {
        $crate::proto::common::option_bool_tokens($entry.$field)
    };
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: str_list,) => {
        $crate::proto::common::str_list_tokens(&$entry.$field)
    };
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: opt_flags,) => {
        $crate::proto::common::option_flags_tokens($entry.$field.as_deref())
    };
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: opt_icon,) => {
        $crate::proto::common::option_icon_tokens($entry.$field.as_deref(), $mod_name)
    };
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: req_icon,) => {{
        let v = $crate::proto::common::resolve_icon_path(&$entry.$field, $mod_name);
        ::quote::quote! { #v }
    }};
    (@tokens_of entry: $entry:ident, mod_name: $mod_name:expr, field: $field:ident, kind: color,) => {
        $crate::proto::common::color_tokens($entry.$field)
    };
}

pub(crate) use define_proto;
