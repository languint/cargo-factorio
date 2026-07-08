use syn::{BinOp, Block, Expr, ExprBinary, Stmt};

use crate::error::{FrontendError, FrontendResult};

use super::{
    context::LowerContext,
    expressions::{lower_assignment_target, lower_expression},
    functions::lower_function,
    types::{infer_type_from_expression, inferred_source_type, lower_binding},
    util::{item_name, location},
};

pub fn lower_block(
    block: &Block,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<factorio_ir::block::Block> {
    let mut statements = Vec::new();
    let last_index = block.stmts.len().saturating_sub(1);

    for (index, statement) in block.stmts.iter().enumerate() {
        let is_tail = index == last_index;
        statements.extend(lower_statement(statement, is_tail, ctx, self_type)?);
    }

    Ok(factorio_ir::block::Block { statements })
}

fn lower_statement(
    statement: &Stmt,
    is_tail: bool,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    match statement {
        Stmt::Local(local) => {
            let (name, annotated_type) = lower_binding(&local.pat)?;
            let init = local
                .init
                .as_ref()
                .ok_or_else(|| FrontendError::MissingLetInitializer {
                    location: location(local),
                })?;
            let value = lower_expression(&init.expr, ctx, self_type)?;
            let (ty, source_type) = if let Some((ty, source_type)) = annotated_type {
                (ty, Some(source_type))
            } else {
                let ty = infer_type_from_expression(&value)
                    .unwrap_or(factorio_ir::r#type::Type::Void);
                let source_type = inferred_source_type(&ty);
                (ty, source_type)
            };

            Ok(vec![factorio_ir::statement::Statement::VariableDecl {
                name,
                ty,
                source_type,
                value,
            }])
        }
        Stmt::Item(syn::Item::Fn(function)) => {
            Ok(vec![factorio_ir::statement::Statement::FunctionDecl(
                lower_function(function, ctx)?,
            )])
        }
        Stmt::Item(item) => Err(FrontendError::UnsupportedItem {
            item: item_name(item),
            location: location(item),
        }),
        Stmt::Expr(expression, semi) => {
            lower_expression_statement(expression, semi.is_some(), is_tail, ctx, self_type)
        }
        Stmt::Macro(mac) => {
            let expression = Expr::Macro(syn::ExprMacro {
                mac: mac.mac.clone(),
                attrs: mac.attrs.clone(),
            });
            lower_expression_statement(&expression, true, is_tail, ctx, self_type)
        }
    }
}

fn lower_expression_statement(
    expression: &Expr,
    has_semi: bool,
    is_tail: bool,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    if has_semi {
        return Ok(vec![lower_semicolon_expression(
            expression, ctx, self_type,
        )?]);
    }

    if is_tail {
        return Ok(vec![lower_tail_expression(expression, ctx, self_type)?]);
    }

    Err(FrontendError::UnsupportedStatement {
        location: location(expression),
    })
}

fn lower_tail_expression(
    expression: &Expr,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<factorio_ir::statement::Statement> {
    match expression {
        Expr::If(if_expression) => lower_if_expression(if_expression, ctx, self_type),
        Expr::Return(return_expression) => Ok(factorio_ir::statement::Statement::Return(
            match return_expression.expr.as_deref() {
                Some(value) => Some(lower_expression(value, ctx, self_type)?),
                None => None,
            },
        )),
        _ => Ok(factorio_ir::statement::Statement::Return(Some(
            lower_expression(expression, ctx, self_type)?,
        ))),
    }
}

fn lower_semicolon_expression(
    expression: &Expr,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<factorio_ir::statement::Statement> {
    match expression {
        Expr::Return(return_expression) => Ok(factorio_ir::statement::Statement::Return(
            match return_expression.expr.as_deref() {
                Some(value) => Some(lower_expression(value, ctx, self_type)?),
                None => None,
            },
        )),
        Expr::Assign(assign) => Ok(lower_assign_statement(assign, ctx, self_type)?),
        Expr::Binary(binary) if is_compound_assign(&binary.op) => {
            Ok(lower_compound_assign_statement(binary, ctx, self_type)?)
        }
        Expr::If(if_expression) => lower_if_expression(if_expression, ctx, self_type),
        Expr::Call(_) | Expr::MethodCall(_) | Expr::Macro(_) => Ok(
            factorio_ir::statement::Statement::Expr(lower_expression(expression, ctx, self_type)?),
        ),
        _ => Err(FrontendError::UnsupportedStatement {
            location: location(expression),
        }),
    }
}

fn lower_assign_statement(
    assign: &syn::ExprAssign,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<factorio_ir::statement::Statement> {
    Ok(factorio_ir::statement::Statement::Assignment {
        target: lower_assignment_target(&assign.left, ctx, self_type)?,
        value: lower_expression(&assign.right, ctx, self_type)?,
    })
}

const fn is_compound_assign(operator: &BinOp) -> bool {
    matches!(
        operator,
        BinOp::AddAssign(_) | BinOp::SubAssign(_) | BinOp::MulAssign(_) | BinOp::DivAssign(_)
    )
}

fn lower_compound_assign_statement(
    binary: &ExprBinary,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<factorio_ir::statement::Statement> {
    let operator = compound_assign_operator(&binary.op)?;
    let target = lower_assignment_target(&binary.left, ctx, self_type)?;
    let rhs = lower_expression(&binary.right, ctx, self_type)?;

    Ok(factorio_ir::statement::Statement::Assignment {
        target: target.clone(),
        value: factorio_ir::expression::Expression::BinaryOp {
            lhs: Box::new(target),
            op: operator,
            rhs: Box::new(rhs),
        },
    })
}

fn compound_assign_operator(operator: &BinOp) -> FrontendResult<factorio_ir::operator::Operator> {
    let operator = match operator {
        BinOp::AddAssign(_) => factorio_ir::operator::Operator::Add,
        BinOp::SubAssign(_) => factorio_ir::operator::Operator::Sub,
        BinOp::MulAssign(_) => factorio_ir::operator::Operator::Mul,
        BinOp::DivAssign(_) => factorio_ir::operator::Operator::Div,
        _ => {
            return Err(FrontendError::UnsupportedOperator {
                location: location(operator),
            });
        }
    };

    Ok(operator)
}

fn lower_if_expression(
    if_expression: &syn::ExprIf,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<factorio_ir::statement::Statement> {
    let condition = lower_expression(&if_expression.cond, ctx, self_type)?;
    let then_block = lower_block_statements(&if_expression.then_branch.stmts, ctx, self_type)?;
    let else_block = match &if_expression.else_branch {
        Some((_, else_branch)) => lower_branch_statements(else_branch, ctx, self_type)?,
        None => Vec::new(),
    };

    Ok(factorio_ir::statement::Statement::Conditional {
        condition,
        then_block,
        else_block,
    })
}

fn lower_branch_statements(
    expression: &Expr,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    match expression {
        Expr::Block(block) => lower_block_statements(&block.block.stmts, ctx, self_type),
        _ => Err(FrontendError::UnsupportedStatement {
            location: location(expression),
        }),
    }
}

fn lower_block_statements(
    statements: &[Stmt],
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    let mut lowered = Vec::new();
    let last_index = statements.len().saturating_sub(1);

    for (index, statement) in statements.iter().enumerate() {
        let is_tail = index == last_index;
        lowered.extend(lower_statement(statement, is_tail, ctx, self_type)?);
    }

    Ok(lowered)
}
