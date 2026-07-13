use syn::{Arm, BinOp, Block, Expr, ExprBinary, ExprMatch, Lit, Pat, Stmt};

use crate::error::{FrontendError, FrontendResult};

use super::{
    context::LowerContext,
    expressions::{lower_assignment_target, lower_expression},
    functions::lower_function,
    print::infer_debug_type_key,
    types::{infer_type_from_expression, inferred_source_type, lower_binding, rust_type_key},
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
            let init = local
                .init
                .as_ref()
                .ok_or_else(|| FrontendError::MissingLetInitializer {
                    location: location(local),
                })?;

            // Handle tuple destructuring: `let (a, b) = (expr_a, expr_b)`
            // Expand into individual `VariableDecl` statements.
            if let Pat::Tuple(pat_tuple) = &local.pat
                && let Expr::Tuple(rhs_tuple) = init.expr.as_ref()
                && pat_tuple.elems.len() == rhs_tuple.elems.len()
            {
                let mut stmts = Vec::new();
                for (pat, rhs) in pat_tuple.elems.iter().zip(rhs_tuple.elems.iter()) {
                    let name = extract_plain_binding(pat).ok_or_else(|| {
                        FrontendError::ExpectedIdentifierPattern {
                            location: location(pat),
                        }
                    })?;
                    let value = lower_expression(rhs, ctx, self_type)?;
                    let ty = infer_type_from_expression(&value)
                        .unwrap_or(factorio_ir::r#type::Type::Void);
                    let source_type = inferred_source_type(&ty);
                    if let Some(key) = infer_debug_type_key(&value, ctx) {
                        ctx.bind_type(name.clone(), key);
                    }
                    stmts.push(factorio_ir::statement::Statement::VariableDecl {
                        name,
                        ty,
                        source_type,
                        value,
                    });
                }
                return Ok(stmts);
            }

            let (name, annotated_type) = lower_binding(&local.pat)?;
            let value = lower_expression(&init.expr, ctx, self_type)?;
            let (ty, source_type) = if let Some((ty, source_type)) = annotated_type {
                (ty, Some(source_type))
            } else {
                let ty =
                    infer_type_from_expression(&value).unwrap_or(factorio_ir::r#type::Type::Void);
                let source_type = inferred_source_type(&ty);
                (ty, source_type)
            };
            if let syn::Pat::Type(pat_type) = &local.pat {
                if let Some(key) = rust_type_key(&pat_type.ty) {
                    ctx.bind_type(name.clone(), key);
                }
            } else if let Some(key) = infer_debug_type_key(&value, ctx) {
                ctx.bind_type(name.clone(), key);
            }

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
    // Block-like expressions always expand to potentially multiple statements
    // (e.g. `if let Some(x) = ...` expands to a VariableDecl + Conditional).
    // Handle them uniformly regardless of tail/semicolon position.
    match expression {
        Expr::If(if_expression) => {
            return lower_if_expression(if_expression, ctx, self_type);
        }
        Expr::ForLoop(for_loop) => {
            return lower_for_loop(for_loop, ctx, self_type);
        }
        Expr::While(while_expr) => {
            return lower_while_loop(while_expr, ctx, self_type);
        }
        Expr::Loop(loop_expr) => {
            return lower_infinite_loop(loop_expr, ctx, self_type);
        }
        Expr::Match(match_expr) => {
            // Tail value-producing `match` becomes an IIFE so arm results can be returned.
            if is_tail && !has_semi {
                return Ok(vec![factorio_ir::statement::Statement::Return(Some(
                    lower_match_expression(match_expr, ctx, self_type)?,
                ))]);
            }
            return lower_match_statements(match_expr, ctx, self_type);
        }
        _ => {}
    }

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
        Expr::If(if_expression) => {
            let stmts = lower_if_expression(if_expression, ctx, self_type)?;
            // If let expansions produce >1 statement; wrap in a no-op Return for the tail slot.
            // In practice the last statement is always a Conditional.
            Ok(stmts
                .into_iter()
                .last()
                .unwrap_or(factorio_ir::statement::Statement::Return(None)))
        }
        Expr::ForLoop(for_loop) => {
            let stmts = lower_for_loop(for_loop, ctx, self_type)?;
            Ok(stmts
                .into_iter()
                .last()
                .unwrap_or(factorio_ir::statement::Statement::Return(None)))
        }
        Expr::While(while_expr) => {
            let stmts = lower_while_loop(while_expr, ctx, self_type)?;
            Ok(stmts
                .into_iter()
                .last()
                .unwrap_or(factorio_ir::statement::Statement::Return(None)))
        }
        Expr::Loop(loop_expr) => {
            let stmts = lower_infinite_loop(loop_expr, ctx, self_type)?;
            Ok(stmts
                .into_iter()
                .last()
                .unwrap_or(factorio_ir::statement::Statement::Return(None)))
        }
        Expr::Match(match_expr) => Ok(factorio_ir::statement::Statement::Return(Some(
            lower_match_expression(match_expr, ctx, self_type)?,
        ))),
        Expr::Return(return_expression) => Ok(factorio_ir::statement::Statement::Return(
            match return_expression.expr.as_deref() {
                Some(value) => Some(lower_expression(value, ctx, self_type)?),
                None => None,
            },
        )),
        Expr::Continue(_) => Ok(factorio_ir::statement::Statement::Continue),
        Expr::Break(break_expr) => lower_break(break_expr),
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
        Expr::If(if_expression) => {
            let stmts = lower_if_expression(if_expression, ctx, self_type)?;
            Ok(stmts
                .into_iter()
                .last()
                .unwrap_or(factorio_ir::statement::Statement::Return(None)))
        }
        Expr::Call(_) | Expr::MethodCall(_) | Expr::Macro(_) => Ok(
            factorio_ir::statement::Statement::Expr(lower_expression(expression, ctx, self_type)?),
        ),
        Expr::Continue(_) => Ok(factorio_ir::statement::Statement::Continue),
        Expr::Break(break_expr) => lower_break(break_expr),
        Expr::ForLoop(for_loop) => {
            let stmts = lower_for_loop(for_loop, ctx, self_type)?;
            Ok(stmts
                .into_iter()
                .last()
                .unwrap_or(factorio_ir::statement::Statement::Return(None)))
        }
        Expr::While(while_expr) => {
            let stmts = lower_while_loop(while_expr, ctx, self_type)?;
            Ok(stmts
                .into_iter()
                .last()
                .unwrap_or(factorio_ir::statement::Statement::Return(None)))
        }
        Expr::Loop(loop_expr) => {
            let stmts = lower_infinite_loop(loop_expr, ctx, self_type)?;
            Ok(stmts
                .into_iter()
                .last()
                .unwrap_or(factorio_ir::statement::Statement::Return(None)))
        }
        Expr::Match(match_expr) => {
            let stmts = lower_match_statements(match_expr, ctx, self_type)?;
            Ok(stmts
                .into_iter()
                .last()
                .unwrap_or(factorio_ir::statement::Statement::Return(None)))
        }
        _ => Err(FrontendError::UnsupportedStatement {
            location: location(expression),
        }),
    }
}

fn lower_for_loop(
    for_loop: &syn::ExprForLoop,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    let var = extract_plain_binding(&for_loop.pat).ok_or_else(|| {
        FrontendError::ExpectedIdentifierPattern {
            location: location(&for_loop.pat),
        }
    })?;
    let iter = lower_expression(&for_loop.expr, ctx, self_type)?;
    let body = lower_block_statements(&for_loop.body.stmts, ctx, self_type)?;
    Ok(vec![factorio_ir::statement::Statement::ForIn {
        var,
        iter,
        body,
    }])
}

fn lower_while_loop(
    while_expr: &syn::ExprWhile,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    let condition = lower_expression(&while_expr.cond, ctx, self_type)?;
    let body = lower_block_statements(&while_expr.body.stmts, ctx, self_type)?;
    Ok(vec![factorio_ir::statement::Statement::While {
        condition,
        body,
    }])
}

fn lower_infinite_loop(
    loop_expr: &syn::ExprLoop,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    let body = lower_block_statements(&loop_expr.body.stmts, ctx, self_type)?;
    Ok(vec![factorio_ir::statement::Statement::While {
        condition: factorio_ir::expression::Expression::Literal(
            factorio_ir::literal::Literal::Bool(true),
        ),
        body,
    }])
}

fn lower_break(
    break_expr: &syn::ExprBreak,
) -> FrontendResult<factorio_ir::statement::Statement> {
    if break_expr.expr.is_some() || break_expr.label.is_some() {
        return Err(FrontendError::UnsupportedExpression {
            location: location(break_expr)
                .with_note("only bare `break` is supported (no value or label)"),
        });
    }
    Ok(factorio_ir::statement::Statement::Break)
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
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    let then_block = lower_block_statements(&if_expression.then_branch.stmts, ctx, self_type)?;
    let else_block = match &if_expression.else_branch {
        Some((_, else_branch)) => lower_branch_statements(else_branch, ctx, self_type)?,
        None => Vec::new(),
    };

    let clauses = flatten_and_clauses(if_expression.cond.as_ref());
    if clauses
        .iter()
        .any(|clause| matches!(clause, CondClause::Let { .. }))
    {
        return lower_let_chain_clauses(&clauses, then_block, else_block, ctx, self_type);
    }

    // Plain `if cond` (no `let` bindings in the condition).
    let condition = lower_expression(&if_expression.cond, ctx, self_type)?;
    Ok(vec![factorio_ir::statement::Statement::Conditional {
        condition,
        then_block,
        else_block,
    }])
}

enum CondClause<'a> {
    /// A normal boolean expression.
    Expr(&'a Expr),
    /// `let Some(name) = expr` / `let name = expr`.
    Let { binding: String, value: &'a Expr },
}

fn flatten_and_clauses(expr: &Expr) -> Vec<CondClause<'_>> {
    match expr {
        Expr::Paren(paren) => flatten_and_clauses(&paren.expr),
        Expr::Binary(binary) if matches!(binary.op, BinOp::And(_)) => {
            let mut clauses = flatten_and_clauses(&binary.left);
            clauses.extend(flatten_and_clauses(&binary.right));
            clauses
        }
        Expr::Let(let_expr) => extract_some_binding(&let_expr.pat).map_or_else(
            || vec![CondClause::Expr(expr)],
            |binding| {
                vec![CondClause::Let {
                    binding,
                    value: let_expr.expr.as_ref(),
                }]
            },
        ),
        other => vec![CondClause::Expr(other)],
    }
}

fn lower_let_chain_clauses(
    clauses: &[CondClause<'_>],
    then_block: Vec<factorio_ir::statement::Statement>,
    else_block: Vec<factorio_ir::statement::Statement>,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    match clauses {
        [] => Ok(then_block),
        [CondClause::Expr(condition), rest @ ..] => {
            let condition = lower_expression(condition, ctx, self_type)?;
            let nested =
                lower_let_chain_clauses(rest, then_block, else_block.clone(), ctx, self_type)?;
            Ok(vec![factorio_ir::statement::Statement::Conditional {
                condition,
                then_block: nested,
                else_block,
            }])
        }
        [CondClause::Let { binding, value }, rest @ ..] => {
            let rhs = lower_expression(value, ctx, self_type)?;
            if let Some(key) = infer_debug_type_key(&rhs, ctx) {
                ctx.bind_type(binding.clone(), key);
            }
            let nested =
                lower_let_chain_clauses(rest, then_block, else_block.clone(), ctx, self_type)?;
            Ok(vec![
                factorio_ir::statement::Statement::VariableDecl {
                    name: binding.clone(),
                    ty: factorio_ir::r#type::Type::Void,
                    source_type: None,
                    value: rhs,
                },
                factorio_ir::statement::Statement::Conditional {
                    condition: factorio_ir::expression::Expression::BinaryOp {
                        lhs: Box::new(factorio_ir::expression::Expression::Identifier(
                            binding.clone(),
                        )),
                        op: factorio_ir::operator::Operator::Ne,
                        rhs: Box::new(factorio_ir::expression::Expression::Literal(
                            factorio_ir::literal::Literal::Nil,
                        )),
                    },
                    then_block: nested,
                    else_block,
                },
            ])
        }
    }
}

/// Extract the inner binding name from `Some(x)` or plain `x` patterns used in `if let`.
fn extract_some_binding(pat: &Pat) -> Option<String> {
    match pat {
        // `if let Some(x) = ...`
        Pat::TupleStruct(ts) => {
            let last = ts.path.segments.last()?;
            if last.ident != "Some" {
                return None;
            }
            let inner = ts.elems.first()?;
            extract_plain_binding(inner)
        }
        // `if let x = ...` (plain binding without wrapper)
        other => extract_plain_binding(other),
    }
}

fn extract_plain_binding(pat: &Pat) -> Option<String> {
    match pat {
        Pat::Ident(ident) => Some(ident.ident.to_string()),
        Pat::Type(pat_type) => extract_plain_binding(&pat_type.pat),
        _ => None,
    }
}

fn lower_branch_statements(
    expression: &Expr,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    match expression {
        Expr::Block(block) => lower_block_statements(&block.block.stmts, ctx, self_type),
        // `else if cond { ... }` is an `Expr::If`, not a block.
        Expr::If(if_expression) => lower_if_expression(if_expression, ctx, self_type),
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

/// Lower `match` in statement position to a temp binding + nested `if`/`else`.
fn lower_match_statements(
    match_expr: &ExprMatch,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    let tmp = match_tmp_name(match_expr);
    let scrutinee = lower_expression(&match_expr.expr, ctx, self_type)?;
    let mut stmts = vec![factorio_ir::statement::Statement::VariableDecl {
        name: tmp.clone(),
        ty: factorio_ir::r#type::Type::Void,
        source_type: None,
        value: scrutinee,
    }];
    stmts.extend(fold_match_arms(
        &tmp,
        &match_expr.arms,
        MatchArmMode::Statement,
        ctx,
        self_type,
    )?);
    Ok(stmts)
}

/// Lower `match` as a value: `(function() ... end)()`.
pub fn lower_match_expression(
    match_expr: &ExprMatch,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<factorio_ir::expression::Expression> {
    let tmp = match_tmp_name(match_expr);
    let scrutinee = lower_expression(&match_expr.expr, ctx, self_type)?;
    let mut body = vec![factorio_ir::statement::Statement::VariableDecl {
        name: tmp.clone(),
        ty: factorio_ir::r#type::Type::Void,
        source_type: None,
        value: scrutinee,
    }];
    body.extend(fold_match_arms(
        &tmp,
        &match_expr.arms,
        MatchArmMode::Value,
        ctx,
        self_type,
    )?);
    Ok(factorio_ir::expression::Expression::Call {
        func: Box::new(factorio_ir::expression::Expression::Closure {
            params: vec![],
            body: factorio_ir::block::Block { statements: body },
        }),
        args: vec![],
    })
}

#[derive(Clone, Copy)]
enum MatchArmMode {
    Statement,
    Value,
}

fn fold_match_arms(
    tmp: &str,
    arms: &[Arm],
    mode: MatchArmMode,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    let scrutinee = factorio_ir::expression::Expression::Identifier(tmp.to_string());
    let mut else_block = Vec::new();
    for arm in arms.iter().rev() {
        let body = lower_match_arm_body(&arm.body, mode, ctx, self_type)?;
        let guard = match &arm.guard {
            Some((_, guard_expr)) => Some(lower_expression(guard_expr, ctx, self_type)?),
            None => None,
        };
        // Top-level `A | B => body` expands to nested arms sharing body/guard so
        // alternatives can bind the same names from different places.
        for alt in flatten_or_patterns(&arm.pat).into_iter().rev() {
            else_block = emit_match_pattern_arm(
                alt,
                &scrutinee,
                guard.clone(),
                body.clone(),
                else_block,
                ctx,
            )?;
        }
    }
    Ok(else_block)
}

fn emit_match_pattern_arm(
    pat: &Pat,
    scrutinee: &factorio_ir::expression::Expression,
    guard: Option<factorio_ir::expression::Expression>,
    body: Vec<factorio_ir::statement::Statement>,
    else_block: Vec<factorio_ir::statement::Statement>,
    ctx: &mut LowerContext<'_>,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    let (condition, bindings) = lower_match_pattern(pat, scrutinee)?;
    let mut then_block = Vec::new();
    for (name, value) in bindings {
        if let Some(key) = infer_debug_type_key(&value, ctx) {
            ctx.bind_type(name.clone(), key);
        }
        then_block.push(factorio_ir::statement::Statement::VariableDecl {
            name,
            ty: factorio_ir::r#type::Type::Void,
            source_type: None,
            value,
        });
    }

    // Guards run after bindings so they can use pattern names. Guard failure
    // falls through to later arms (same else_block as a pattern miss).
    if let Some(guard) = guard {
        then_block.push(factorio_ir::statement::Statement::Conditional {
            condition: guard,
            then_block: body,
            else_block: else_block.clone(),
        });
    } else {
        then_block.extend(body);
    }

    Ok(match condition {
        None => then_block,
        Some(condition) => {
            vec![factorio_ir::statement::Statement::Conditional {
                condition,
                then_block,
                else_block,
            }]
        }
    })
}

fn lower_match_arm_body(
    body: &Expr,
    mode: MatchArmMode,
    ctx: &mut LowerContext<'_>,
    self_type: Option<&str>,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    match mode {
        MatchArmMode::Statement => match body {
            Expr::Block(block) => lower_block_statements(&block.block.stmts, ctx, self_type),
            other => Ok(vec![lower_semicolon_expression(other, ctx, self_type)?]),
        },
        MatchArmMode::Value => match body {
            Expr::Block(block) => Ok(lower_block(&block.block, ctx, self_type)?.statements),
            other => Ok(vec![factorio_ir::statement::Statement::Return(Some(
                lower_expression(other, ctx, self_type)?,
            ))]),
        },
    }
}

fn flatten_or_patterns(pat: &Pat) -> Vec<&Pat> {
    match pat {
        Pat::Or(or_pat) => or_pat
            .cases
            .iter()
            .flat_map(flatten_or_patterns)
            .collect(),
        Pat::Paren(paren) => flatten_or_patterns(&paren.pat),
        other => vec![other],
    }
}

type MatchPatternParts = (
    Option<factorio_ir::expression::Expression>,
    Vec<(String, factorio_ir::expression::Expression)>,
);

/// Returns `(condition, bindings)`. `None` condition is irrefutable (`_` / plain ident).
fn lower_match_pattern(
    pat: &Pat,
    scrutinee: &factorio_ir::expression::Expression,
) -> FrontendResult<MatchPatternParts> {
    match pat {
        Pat::Wild(_) => Ok((None, vec![])),
        Pat::Ident(ident) if ident.ident == "None" => Ok((Some(eq_nil(scrutinee.clone())), vec![])),
        Pat::Ident(ident) => Ok((
            None,
            vec![(ident.ident.to_string(), scrutinee.clone())],
        )),
        Pat::Lit(lit) => Ok((
            Some(eq_expr(scrutinee.clone(), literal_from_pat_lit(lit, pat)?)),
            vec![],
        )),
        Pat::Path(path) if is_none_path(&path.path) => {
            Ok((Some(eq_nil(scrutinee.clone())), vec![]))
        }
        Pat::TupleStruct(ts) if is_some_path(&ts.path) => {
            let inner = ts.elems.first().ok_or_else(|| FrontendError::UnsupportedExpression {
                location: location(pat).with_note("Some(...) pattern requires one binding"),
            })?;
            let (inner_cond, inner_binds) = lower_match_pattern(inner, scrutinee)?;
            Ok((
                Some(and_conditions(ne_nil(scrutinee.clone()), inner_cond)),
                inner_binds,
            ))
        }
        Pat::Struct(struct_pat) => lower_struct_pattern(struct_pat, scrutinee),
        Pat::Or(or_pat) => lower_nested_or_pattern(or_pat, scrutinee),
        Pat::Type(pat_type) => lower_match_pattern(&pat_type.pat, scrutinee),
        Pat::Paren(paren) => lower_match_pattern(&paren.pat, scrutinee),
        Pat::Reference(reference) => lower_match_pattern(&reference.pat, scrutinee),
        _ => Err(FrontendError::UnsupportedExpression {
            location: location(pat).with_note(
                "match supports `_`, literals, `None`, `Some(...)`, struct patterns, or-patterns, and plain bindings",
            ),
        }),
    }
}

fn lower_struct_pattern(
    struct_pat: &syn::PatStruct,
    scrutinee: &factorio_ir::expression::Expression,
) -> FrontendResult<MatchPatternParts> {
    let mut condition = None;
    let mut bindings = Vec::new();
    for field in &struct_pat.fields {
        let field_name = match &field.member {
            syn::Member::Named(ident) => ident.to_string(),
            syn::Member::Unnamed(index) => {
                return Err(FrontendError::UnsupportedExpression {
                    location: location(index)
                        .with_note("tuple struct field patterns are not supported"),
                });
            }
        };
        let field_scrutinee = factorio_ir::expression::Expression::FieldAccess {
            base: Box::new(scrutinee.clone()),
            field: field_name,
        };
        let (field_cond, field_binds) = lower_match_pattern(&field.pat, &field_scrutinee)?;
        condition = match (condition, field_cond) {
            (None, c) => c,
            (Some(left), Some(right)) => Some(and_expr(left, right)),
            (Some(left), None) => Some(left),
        };
        bindings.extend(field_binds);
    }
    // `..` / rest is ignored — Lua tables have no exhaustiveness check.
    Ok((condition, bindings))
}

fn lower_nested_or_pattern(
    or_pat: &syn::PatOr,
    scrutinee: &factorio_ir::expression::Expression,
) -> FrontendResult<MatchPatternParts> {
    let mut condition = None;
    let mut any_irrefutable = false;
    let mut bindings: Option<Vec<(String, factorio_ir::expression::Expression)>> = None;
    for case in &or_pat.cases {
        let (case_cond, case_binds) = lower_match_pattern(case, scrutinee)?;
        match &mut bindings {
            None => bindings = Some(case_binds),
            Some(existing) => {
                if existing != &case_binds {
                    return Err(FrontendError::UnsupportedExpression {
                        location: location(or_pat).with_note(
                            "nested or-pattern alternatives must bind the same names the same way; use separate match arms",
                        ),
                    });
                }
            }
        }
        match case_cond {
            None => any_irrefutable = true,
            Some(case_cond) => {
                condition = Some(match condition {
                    Some(left) => or_expr(left, case_cond),
                    None => case_cond,
                });
            }
        }
    }
    let condition = if any_irrefutable { None } else { condition };
    Ok((condition, bindings.unwrap_or_default()))
}

fn literal_from_pat_lit(
    lit: &syn::PatLit,
    pat: &Pat,
) -> FrontendResult<factorio_ir::expression::Expression> {
    let literal = match &lit.lit {
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
                location: location(pat).with_note("unsupported match literal"),
            });
        }
    };
    Ok(factorio_ir::expression::Expression::Literal(literal))
}

fn eq_expr(
    lhs: factorio_ir::expression::Expression,
    rhs: factorio_ir::expression::Expression,
) -> factorio_ir::expression::Expression {
    factorio_ir::expression::Expression::BinaryOp {
        lhs: Box::new(lhs),
        op: factorio_ir::operator::Operator::Eq,
        rhs: Box::new(rhs),
    }
}

fn eq_nil(expr: factorio_ir::expression::Expression) -> factorio_ir::expression::Expression {
    eq_expr(
        expr,
        factorio_ir::expression::Expression::Literal(factorio_ir::literal::Literal::Nil),
    )
}

fn ne_nil(expr: factorio_ir::expression::Expression) -> factorio_ir::expression::Expression {
    factorio_ir::expression::Expression::BinaryOp {
        lhs: Box::new(expr),
        op: factorio_ir::operator::Operator::Ne,
        rhs: Box::new(factorio_ir::expression::Expression::Literal(
            factorio_ir::literal::Literal::Nil,
        )),
    }
}

fn and_expr(
    lhs: factorio_ir::expression::Expression,
    rhs: factorio_ir::expression::Expression,
) -> factorio_ir::expression::Expression {
    factorio_ir::expression::Expression::BinaryOp {
        lhs: Box::new(lhs),
        op: factorio_ir::operator::Operator::And,
        rhs: Box::new(rhs),
    }
}

fn and_conditions(
    left: factorio_ir::expression::Expression,
    right: Option<factorio_ir::expression::Expression>,
) -> factorio_ir::expression::Expression {
    match right {
        Some(right) => and_expr(left, right),
        None => left,
    }
}

fn or_expr(
    lhs: factorio_ir::expression::Expression,
    rhs: factorio_ir::expression::Expression,
) -> factorio_ir::expression::Expression {
    factorio_ir::expression::Expression::BinaryOp {
        lhs: Box::new(lhs),
        op: factorio_ir::operator::Operator::Or,
        rhs: Box::new(rhs),
    }
}

fn is_none_path(path: &syn::Path) -> bool {
    path.segments
        .last()
        .is_some_and(|seg| seg.ident == "None")
}

fn is_some_path(path: &syn::Path) -> bool {
    path.segments
        .last()
        .is_some_and(|seg| seg.ident == "Some")
}

fn match_tmp_name(match_expr: &ExprMatch) -> String {
    use syn::spanned::Spanned;
    format!("__match_{}", match_expr.span().byte_range().start)
}
