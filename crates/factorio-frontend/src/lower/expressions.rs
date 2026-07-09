use syn::{BinOp, Expr, ExprBinary, ExprLit, ExprPath, Lit, Member};

use crate::error::{FrontendError, FrontendResult};

use super::{context::LowerContext, print::lower_macro_expression, util::location};

pub fn lower_expression(
    expression: &Expr,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<factorio_ir::expression::Expression> {
    match expression {
        Expr::Binary(binary) => lower_binary_expression(binary, ctx, self_type),
        Expr::Lit(literal) => lower_literal_expression(literal),
        Expr::Path(path) => lower_path_expression(path, ctx, self_type),
        Expr::Field(field) => lower_field_expression(field, ctx, self_type),
        Expr::Call(call) => {
            let func = lower_expression(&call.func, ctx, self_type)?;
            let args = call
                .args
                .iter()
                .map(|arg| lower_expression(arg, ctx, self_type))
                .collect::<FrontendResult<Vec<_>>>()?;
            Ok(factorio_ir::expression::Expression::Call {
                func: Box::new(func),
                args,
            })
        }
        Expr::MethodCall(call) => {
            let receiver = lower_expression(&call.receiver, ctx, self_type)?;
            let args = call
                .args
                .iter()
                .map(|arg| lower_expression(arg, ctx, self_type))
                .collect::<FrontendResult<Vec<_>>>()?;
            Ok(factorio_ir::expression::Expression::MethodCall {
                receiver: Box::new(receiver),
                method: call.method.to_string(),
                args,
            })
        }
        Expr::Struct(item) => lower_struct_expression(item, ctx, self_type),
        Expr::Macro(mac) => lower_macro_expression(mac, ctx, self_type),
        _ => Err(FrontendError::UnsupportedExpression {
            location: location(expression),
        }),
    }
}

pub fn lower_assignment_target(
    expression: &Expr,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<factorio_ir::expression::Expression> {
    match expression {
        Expr::Path(path) => lower_path_expression(path, ctx, self_type),
        Expr::Field(field) => lower_field_expression(field, ctx, self_type),
        _ => Err(FrontendError::ExpectedIdentifierAssignmentTarget {
            location: location(expression),
        }),
    }
}

fn lower_struct_expression(
    item: &syn::ExprStruct,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<factorio_ir::expression::Expression> {
    let fields = item
        .fields
        .iter()
        .map(|field| {
            let name = match &field.member {
                Member::Named(ident) => ident.to_string(),
                Member::Unnamed(index) => {
                    return Err(FrontendError::UnsupportedExpression {
                        location: location(index),
                    });
                }
            };
            Ok((name, lower_expression(&field.expr, ctx, self_type)?))
        })
        .collect::<FrontendResult<Vec<_>>>()?;

    Ok(factorio_ir::expression::Expression::StructLiteral { fields })
}

fn lower_field_expression(
    field: &syn::ExprField,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<factorio_ir::expression::Expression> {
    let base = lower_expression(&field.base, ctx, self_type)?;
    let field_name = match &field.member {
        Member::Named(ident) => ident.to_string(),
        Member::Unnamed(index) => {
            return Err(FrontendError::UnsupportedExpression {
                location: location(index),
            });
        }
    };

    Ok(factorio_ir::expression::Expression::FieldAccess {
        base: Box::new(base),
        field: field_name,
    })
}

fn lower_binary_expression(
    binary: &ExprBinary,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<factorio_ir::expression::Expression> {
    let lhs = lower_expression(&binary.left, ctx, self_type)?;
    let op = lower_binary_operator(&binary.op)?;
    let rhs = lower_expression(&binary.right, ctx, self_type)?;

    Ok(factorio_ir::expression::Expression::BinaryOp {
        lhs: Box::new(lhs),
        op,
        rhs: Box::new(rhs),
    })
}

fn lower_binary_operator(operator: &BinOp) -> FrontendResult<factorio_ir::operator::Operator> {
    let operator = match operator {
        BinOp::Add(_) => factorio_ir::operator::Operator::Add,
        BinOp::Sub(_) => factorio_ir::operator::Operator::Sub,
        BinOp::Mul(_) => factorio_ir::operator::Operator::Mul,
        BinOp::Div(_) => factorio_ir::operator::Operator::Div,
        BinOp::Eq(_) => factorio_ir::operator::Operator::Eq,
        BinOp::Ne(_) => factorio_ir::operator::Operator::Ne,
        BinOp::Lt(_) => factorio_ir::operator::Operator::Lt,
        BinOp::Le(_) => factorio_ir::operator::Operator::Le,
        BinOp::Gt(_) => factorio_ir::operator::Operator::Gt,
        BinOp::Ge(_) => factorio_ir::operator::Operator::Ge,
        _ => {
            return Err(FrontendError::UnsupportedOperator {
                location: location(operator),
            });
        }
    };

    Ok(operator)
}

fn lower_literal_expression(
    literal: &ExprLit,
) -> FrontendResult<factorio_ir::expression::Expression> {
    let literal = match &literal.lit {
        Lit::Int(value) => {
            let parsed = value
                .base10_parse::<i64>()
                .map_err(|error| FrontendError::Syn(format!("invalid integer literal: {error}")))?;
            factorio_ir::literal::Literal::Int(parsed)
        }
        Lit::Float(value) => {
            let parsed = value
                .base10_parse::<f64>()
                .map_err(|error| FrontendError::Syn(format!("invalid float literal: {error}")))?;
            factorio_ir::literal::Literal::Float(parsed)
        }
        Lit::Str(value) => factorio_ir::literal::Literal::String(value.value()),
        Lit::Bool(value) => factorio_ir::literal::Literal::Bool(value.value),
        _ => {
            return Err(FrontendError::UnsupportedExpression {
                location: location(literal),
            });
        }
    };

    Ok(factorio_ir::expression::Expression::Literal(literal))
}

fn lower_path_expression(
    path: &ExprPath,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<factorio_ir::expression::Expression> {
    let mut segments = lower_path_segments(path, self_type)?;
    ctx.normalize_crate_path(&mut segments)?;

    // Map Rust Option/bool keywords to Lua literals.
    if segments.len() == 1 {
        match segments[0].as_str() {
            "None" => {
                return Ok(factorio_ir::expression::Expression::Literal(
                    factorio_ir::literal::Literal::Nil,
                ));
            }
            "true" | "false" => {
                unreachable!("bool literals are handled by lower_literal_expression")
            }
            _ => {}
        }
    }

    match segments.len() {
        1 => Ok(factorio_ir::expression::Expression::Identifier(
            segments[0].clone(),
        )),
        _ => Ok(factorio_ir::expression::Expression::QualifiedPath { segments }),
    }
}

fn lower_path_segments(path: &ExprPath, self_type: Option<&str>) -> FrontendResult<Vec<String>> {
    path.path
        .segments
        .iter()
        .map(|segment| resolve_path_segment(&segment.ident, self_type))
        .collect()
}

fn resolve_path_segment(ident: &syn::Ident, self_type: Option<&str>) -> FrontendResult<String> {
    if ident == "Self" {
        return self_type
            .map(str::to_string)
            .ok_or_else(|| FrontendError::UnsupportedExpression {
                location: location(ident),
            });
    }

    Ok(ident.to_string())
}
