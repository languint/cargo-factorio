use syn::{GenericArgument, ImplItem, PathArguments, Type, TypeParamBound};

use crate::error::{FrontendError, FrontendResult};

use super::{
    context::LowerContext,
    functions::lower_impl_method,
    structs::{PendingEnum, PendingStruct},
    util::location,
};

pub struct FromConversion {
    #[allow(dead_code)] // reserved for typed call-site coercion
    pub target: String,
}

/// First type argument of `Into<T>` / `From<T>` when present.
#[must_use]
pub fn convert_generic_arg(path: &syn::Path) -> Option<String> {
    let segment = path.segments.last()?;
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    for arg in &args.args {
        if let GenericArgument::Type(ty) = arg {
            return path_type_name(ty);
        }
    }
    None
}

/// `impl Into<Widget>` / `impl Into<crate::Widget>` -> `Widget`.
#[must_use]
pub fn into_target_type(ty: &Type) -> Option<String> {
    let Type::ImplTrait(impl_trait) = ty else {
        return None;
    };
    for bound in &impl_trait.bounds {
        let TypeParamBound::Trait(trait_bound) = bound else {
            continue;
        };
        let segment = trait_bound.path.segments.last()?;
        if segment.ident == "Into" {
            return convert_generic_arg(&trait_bound.path);
        }
    }
    None
}

/// `From<Frame>` path -> `Frame`.
#[must_use]
pub fn from_source_type(trait_path: &syn::Path) -> Option<String> {
    let segment = trait_path.segments.last()?;
    if segment.ident != "From" {
        return None;
    }
    convert_generic_arg(trait_path)
}

/// Trait path ends with `From` (allows `From` / `convert::From` / `core::convert::From`).
#[must_use]
pub fn is_from_trait(trait_path: &syn::Path) -> bool {
    trait_path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "From")
}

fn path_type_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(path) => path.path.segments.last().map(|s| s.ident.to_string()),
        Type::Reference(reference) => path_type_name(&reference.elem),
        Type::Paren(paren) => path_type_name(&paren.elem),
        Type::Group(group) => path_type_name(&group.elem),
        _ => None,
    }
}

/// Lower `impl From<Source> for Target { fn from(...) }`.
pub fn lower_from_impl(
    item_impl: &syn::ItemImpl,
    source: &str,
    target: &str,
    structs: &mut std::collections::BTreeMap<String, PendingStruct>,
    enums: &mut std::collections::BTreeMap<String, PendingEnum>,
    ctx: &mut LowerContext<'_>,
) -> FrontendResult<()> {
    let mut from_fn = None;
    for impl_item in &item_impl.items {
        match impl_item {
            ImplItem::Fn(method) if method.sig.ident == "from" => {
                from_fn = Some(method);
            }
            ImplItem::Fn(method) => {
                return Err(FrontendError::UnsupportedItem {
                    item: format!(
                        "method `{}` in `impl From<{source}> for {target}` (only `from` is supported)",
                        method.sig.ident
                    ),
                    location: location(method),
                });
            }
            ImplItem::Type(item) => {
                return Err(FrontendError::UnsupportedItem {
                    item: format!("associated type in From impl (`{}`)", item.ident),
                    location: location(item),
                });
            }
            item => {
                return Err(FrontendError::UnsupportedItem {
                    item: super::util::item_name_impl(item),
                    location: location(item),
                });
            }
        }
    }
    let Some(from_fn) = from_fn else {
        return Err(FrontendError::UnsupportedItem {
            item: format!("`impl From<{source}> for {target}` is missing `fn from`"),
            location: location(item_impl),
        });
    };

    let old_param = from_fn
        .sig
        .inputs
        .iter()
        .find_map(|input| match input {
            syn::FnArg::Typed(pat_type) => match pat_type.pat.as_ref() {
                syn::Pat::Ident(ident) => Some(ident.ident.to_string()),
                _ => None,
            },
            syn::FnArg::Receiver(_) => None,
        })
        .unwrap_or_else(|| "value".to_string());

    // Lower `from` against the target type (`Self` in the body).
    let mut into_method = lower_impl_method(from_fn, target, ctx)?;
    into_method.name = "into".to_string();
    if let Some(param) = into_method.params.first_mut() {
        param.name = "self".to_string();
        param.source_type = Some("self".to_string());
    }
    rename_identifier_in_block(&mut into_method.body.statements, &old_param, "self");

    push_into_method(source, into_method, structs, enums, item_impl)?;

    ctx.from_conversions.insert(
        source.to_string(),
        FromConversion {
            target: target.to_string(),
        },
    );

    ensure_identity_into(target, structs, enums);
    Ok(())
}

fn push_into_method(
    source: &str,
    into_method: factorio_ir::function::Function,
    structs: &mut std::collections::BTreeMap<String, PendingStruct>,
    enums: &mut std::collections::BTreeMap<String, PendingEnum>,
    item_impl: &syn::ItemImpl,
) -> FrontendResult<()> {
    if let Some(entry) = enums.get_mut(source) {
        if entry.methods.iter().any(|m| m.name == "into") {
            return Err(duplicate_into(source, item_impl));
        }
        entry.methods.push(into_method);
    } else {
        let entry = structs
            .entry(source.to_string())
            .or_insert_with(|| PendingStruct::new(syn::Visibility::Inherited));
        if entry.methods.iter().any(|m| m.name == "into") {
            return Err(duplicate_into(source, item_impl));
        }
        entry.methods.push(into_method);
    }
    Ok(())
}

fn duplicate_into(source: &str, item_impl: &syn::ItemImpl) -> FrontendError {
    FrontendError::UnsupportedItem {
        item: format!(
            "type `{source}` already has `into` (only one `impl From<{source}> for _` is supported)"
        ),
        location: location(item_impl),
    }
}

fn ensure_identity_into(
    target: &str,
    structs: &mut std::collections::BTreeMap<String, PendingStruct>,
    enums: &mut std::collections::BTreeMap<String, PendingEnum>,
) {
    let has_into = enums
        .get(target)
        .is_some_and(|e| e.methods.iter().any(|m| m.name == "into"))
        || structs
            .get(target)
            .is_some_and(|e| e.methods.iter().any(|m| m.name == "into"));
    if has_into {
        return;
    }

    let identity = factorio_ir::function::Function {
        name: "into".to_string(),
        params: vec![factorio_ir::function::Parameter {
            name: "self".to_string(),
            r#type: factorio_ir::r#type::Type::Void,
            source_type: Some("self".to_string()),
        }],
        body: factorio_ir::block::Block {
            statements: vec![factorio_ir::statement::Statement::Return(Some(
                factorio_ir::expression::Expression::Identifier("self".to_string()),
            ))],
        },
        doc: Some(format!(
            "Identity `into` for `{target}` (already converted)."
        )),
        debug: None,
        event: None,
        event_filter: None,
        export: None,
        inline: false,
    };

    if let Some(entry) = enums.get_mut(target) {
        entry.methods.push(identity);
    } else {
        structs
            .entry(target.to_string())
            .or_insert_with(|| PendingStruct::new(syn::Visibility::Inherited))
            .methods
            .push(identity);
    }
}

fn rename_identifier_in_block(
    statements: &mut [factorio_ir::statement::Statement],
    from: &str,
    to: &str,
) {
    for statement in statements.iter_mut() {
        rename_identifier_in_statement(statement, from, to);
    }
}

fn rename_identifier_in_statement(
    statement: &mut factorio_ir::statement::Statement,
    from: &str,
    to: &str,
) {
    use factorio_ir::statement::Statement;
    match statement {
        Statement::Return(Some(expr)) | Statement::Expr(expr) => {
            rename_identifier_in_expr(expr, from, to);
        }
        Statement::VariableDecl { name, value, .. } => {
            rename_identifier_in_expr(value, from, to);
            if name == from {
                *name = to.to_string();
            }
        }
        Statement::Assignment { target, value } => {
            rename_identifier_in_expr(target, from, to);
            rename_identifier_in_expr(value, from, to);
        }
        Statement::Conditional {
            condition,
            then_block,
            else_block,
        } => {
            rename_identifier_in_expr(condition, from, to);
            rename_identifier_in_block(then_block, from, to);
            rename_identifier_in_block(else_block, from, to);
        }
        Statement::While { condition, body } => {
            rename_identifier_in_expr(condition, from, to);
            rename_identifier_in_block(body, from, to);
        }
        Statement::ForIn { iter, body, .. } => {
            rename_identifier_in_expr(iter, from, to);
            rename_identifier_in_block(body, from, to);
        }
        Statement::ForNumeric {
            start, limit, body, ..
        } => {
            rename_identifier_in_expr(start, from, to);
            rename_identifier_in_expr(limit, from, to);
            rename_identifier_in_block(body, from, to);
        }
        Statement::FunctionDecl(function) => {
            rename_identifier_in_block(&mut function.body.statements, from, to);
        }
        _ => {}
    }
}

fn rename_identifier_in_expr(expr: &mut factorio_ir::expression::Expression, from: &str, to: &str) {
    use factorio_ir::expression::Expression;
    match expr {
        Expression::Identifier(name) if name == from => {
            *name = to.to_string();
        }
        Expression::FieldAccess { base, .. }
        | Expression::Not(base)
        | Expression::Len(base)
        | Expression::FatPointer { data: base, .. } => {
            rename_identifier_in_expr(base, from, to);
        }
        Expression::BinaryOp { lhs, rhs, .. } => {
            rename_identifier_in_expr(lhs, from, to);
            rename_identifier_in_expr(rhs, from, to);
        }
        Expression::Call { func, args } => {
            rename_identifier_in_expr(func, from, to);
            for arg in args {
                rename_identifier_in_expr(arg, from, to);
            }
        }
        Expression::MethodCall { receiver, args, .. }
        | Expression::DynMethodCall { receiver, args, .. } => {
            rename_identifier_in_expr(receiver, from, to);
            for arg in args {
                rename_identifier_in_expr(arg, from, to);
            }
        }
        Expression::StructLiteral { fields, .. } | Expression::EnumLiteral { fields, .. } => {
            for (_, value) in fields {
                rename_identifier_in_expr(value, from, to);
            }
        }
        Expression::Array { elements } | Expression::FormatConcat { parts: elements } => {
            for element in elements {
                rename_identifier_in_expr(element, from, to);
            }
        }
        Expression::Index { base, key } => {
            rename_identifier_in_expr(base, from, to);
            rename_identifier_in_expr(key, from, to);
        }
        Expression::If {
            condition,
            then_expr,
            else_expr,
        } => {
            rename_identifier_in_expr(condition, from, to);
            rename_identifier_in_expr(then_expr, from, to);
            rename_identifier_in_expr(else_expr, from, to);
        }
        Expression::Closure { body, .. } => {
            rename_identifier_in_block(&mut body.statements, from, to);
        }
        _ => {}
    }
}

/// Whether `.into()` on this receiver should be a real method call.
pub fn into_should_call_method(receiver: &syn::Expr, ctx: &LowerContext<'_>) -> bool {
    match receiver {
        syn::Expr::Path(path) if path.path.segments.len() == 1 => {
            let name = path.path.segments[0].ident.to_string();
            if ctx.into_params.contains(&name) {
                return true;
            }
            if let Some(key) = ctx.binding_type(&name)
                && ctx.from_conversions.contains_key(key)
            {
                return true;
            }
            false
        }
        syn::Expr::MethodCall(_) | syn::Expr::Call(_) | syn::Expr::Struct(_) => {
            super::traits::resolve_concrete_type(receiver, ctx)
                .is_some_and(|ty| ctx.from_conversions.contains_key(&ty))
        }
        syn::Expr::Paren(paren) => into_should_call_method(&paren.expr, ctx),
        syn::Expr::Reference(reference) => into_should_call_method(&reference.expr, ctx),
        syn::Expr::Group(group) => into_should_call_method(&group.expr, ctx),
        _ => false,
    }
}

/// Record `impl Into<_>` parameter names so `.into()` on them is a method call.
pub fn bind_into_param(ctx: &mut LowerContext<'_>, name: String) {
    ctx.into_params.insert(name);
}
