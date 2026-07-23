use crate::{
    block::Block,
    expression::Expression,
    function::Function,
    literal::Literal,
    module::{Module, Symbol},
    statement::Statement,
};

pub(super) fn optimize_module(module: &mut Module) {
    optimize_block(&mut module.body);
    for symbol in &mut module.symbols {
        optimize_symbol(symbol);
    }
}

fn optimize_symbol(symbol: &mut Symbol) {
    optimize_statement_inplace(&mut symbol.statement);
}

fn optimize_block(block: &mut Block) {
    block.statements = optimize_statements(std::mem::take(&mut block.statements));
}

fn optimize_statement_inplace(statement: &mut Statement) {
    match statement {
        Statement::FunctionDecl(function) => optimize_function(function),
        Statement::StructDecl(struct_decl) => {
            for method in &mut struct_decl.methods {
                optimize_function(method);
            }
        }
        Statement::EnumDecl(enum_decl) => {
            for method in &mut enum_decl.methods {
                optimize_function(method);
            }
            for (_, value) in &mut enum_decl.constants {
                optimize_expression(value);
            }
        }
        Statement::VariableDecl { value, .. }
        | Statement::Return(Some(value))
        | Statement::Expr(value) => optimize_expression(value),
        Statement::Assignment { target, value } => {
            optimize_expression(target);
            optimize_expression(value);
        }
        Statement::Conditional {
            condition,
            then_block,
            else_block,
        } => {
            optimize_expression(condition);
            *then_block = optimize_statements(std::mem::take(then_block));
            *else_block = optimize_statements(std::mem::take(else_block));
        }
        Statement::Return(None) | Statement::Continue | Statement::Break => {}
        Statement::ForIn { iter, body, .. } => {
            optimize_expression(iter);
            *body = optimize_statements(std::mem::take(body));
        }
        Statement::ForNumeric {
            start, limit, body, ..
        } => {
            optimize_expression(start);
            optimize_expression(limit);
            *body = optimize_statements(std::mem::take(body));
        }
        Statement::While { condition, body } => {
            optimize_expression(condition);
            *body = optimize_statements(std::mem::take(body));
        }
    }
}

fn optimize_function(function: &mut Function) {
    optimize_block(&mut function.body);
    if let Some(filter) = &mut function.event_filter {
        optimize_expression(filter);
    }
}

fn optimize_statements(statements: Vec<Statement>) -> Vec<Statement> {
    let mut out = Vec::with_capacity(statements.len());
    for statement in statements {
        out.extend(expand_statement(statement));
    }
    out
}

fn expand_statement(mut statement: Statement) -> Vec<Statement> {
    optimize_statement_inplace(&mut statement);

    match statement {
        Statement::VariableDecl {
            name,
            ty,
            source_type,
            value,
        } => expand_value_binding(
            ValueSink::Local {
                name,
                ty,
                source_type,
            },
            value,
        ),
        Statement::Assignment { target, value } if is_simple_assign_target(&target) => {
            expand_value_binding(ValueSink::Assign { target }, value)
        }
        Statement::Return(Some(value)) => expand_value_binding(ValueSink::Return, value),
        other => vec![other],
    }
}

enum ValueSink {
    Local {
        name: String,
        ty: crate::r#type::Type,
        source_type: Option<String>,
    },
    Assign {
        target: Expression,
    },
    Return,
}

fn expand_value_binding(sink: ValueSink, value: Expression) -> Vec<Statement> {
    match value {
        Expression::If {
            condition,
            then_expr,
            else_expr,
        } => expand_if(sink, *condition, *then_expr, *else_expr),
        Expression::Call { func, args } if args.is_empty() => match *func {
            Expression::Closure { params, body } if params.is_empty() => expand_iife(sink, body),
            other => finish_sink(
                sink,
                Expression::Call {
                    func: Box::new(other),
                    args,
                },
            ),
        },
        value => finish_sink(sink, value),
    }
}

fn finish_sink(sink: ValueSink, value: Expression) -> Vec<Statement> {
    match sink {
        ValueSink::Local {
            name,
            ty,
            source_type,
        } => vec![Statement::VariableDecl {
            name,
            ty,
            source_type,
            value,
        }],
        ValueSink::Assign { target } => vec![Statement::Assignment { target, value }],
        ValueSink::Return => vec![Statement::Return(Some(value))],
    }
}

fn expand_if(
    sink: ValueSink,
    condition: Expression,
    then_expr: Expression,
    else_expr: Expression,
) -> Vec<Statement> {
    match sink {
        ValueSink::Local {
            name,
            ty,
            source_type,
        } => {
            let then_block = optimize_statements(vec![Statement::Assignment {
                target: Expression::Identifier(name.clone()),
                value: then_expr,
            }]);
            let else_block = optimize_statements(vec![Statement::Assignment {
                target: Expression::Identifier(name.clone()),
                value: else_expr,
            }]);
            vec![
                Statement::VariableDecl {
                    name,
                    ty,
                    source_type,
                    value: Expression::Literal(Literal::Nil),
                },
                Statement::Conditional {
                    condition,
                    then_block,
                    else_block,
                },
            ]
        }
        ValueSink::Assign { target } => {
            let then_block = optimize_statements(vec![Statement::Assignment {
                target: target.clone(),
                value: then_expr,
            }]);
            let else_block = optimize_statements(vec![Statement::Assignment {
                target,
                value: else_expr,
            }]);
            vec![Statement::Conditional {
                condition,
                then_block,
                else_block,
            }]
        }
        ValueSink::Return => {
            let then_block = optimize_statements(vec![Statement::Return(Some(then_expr))]);
            let else_block = optimize_statements(vec![Statement::Return(Some(else_expr))]);
            vec![Statement::Conditional {
                condition,
                then_block,
                else_block,
            }]
        }
    }
}

fn expand_iife(sink: ValueSink, body: Block) -> Vec<Statement> {
    match sink {
        ValueSink::Local {
            name,
            ty,
            source_type,
        } => {
            let mut out = vec![Statement::VariableDecl {
                name: name.clone(),
                ty,
                source_type,
                value: Expression::Literal(Literal::Nil),
            }];
            out.extend(optimize_statements(remap_returns_to_assign(
                body.statements,
                &Expression::Identifier(name),
            )));
            out
        }
        ValueSink::Assign { target } => {
            optimize_statements(remap_returns_to_assign(body.statements, &target))
        }
        ValueSink::Return => optimize_statements(body.statements),
    }
}

fn remap_returns_to_assign(statements: Vec<Statement>, target: &Expression) -> Vec<Statement> {
    statements
        .into_iter()
        .map(|statement| remap_return_statement(statement, target))
        .collect()
}

fn remap_return_statement(statement: Statement, target: &Expression) -> Statement {
    match statement {
        Statement::Return(value) => Statement::Assignment {
            target: target.clone(),
            value: value.unwrap_or(Expression::Literal(Literal::Nil)),
        },
        Statement::Conditional {
            condition,
            then_block,
            else_block,
        } => Statement::Conditional {
            condition,
            then_block: then_block
                .into_iter()
                .map(|s| remap_return_statement(s, target))
                .collect(),
            else_block: else_block
                .into_iter()
                .map(|s| remap_return_statement(s, target))
                .collect(),
        },
        Statement::ForIn {
            var,
            iter,
            body,
            ipairs,
        } => Statement::ForIn {
            var,
            iter,
            body: body
                .into_iter()
                .map(|s| remap_return_statement(s, target))
                .collect(),
            ipairs,
        },
        Statement::ForNumeric {
            var,
            start,
            limit,
            body,
        } => Statement::ForNumeric {
            var,
            start,
            limit,
            body: body
                .into_iter()
                .map(|s| remap_return_statement(s, target))
                .collect(),
        },
        Statement::While { condition, body } => Statement::While {
            condition,
            body: body
                .into_iter()
                .map(|s| remap_return_statement(s, target))
                .collect(),
        },
        // Nested function values keep their own returns.
        other => other,
    }
}

const fn is_simple_assign_target(target: &Expression) -> bool {
    matches!(
        target,
        Expression::Identifier(_)
            | Expression::FieldAccess { .. }
            | Expression::Index { .. }
            | Expression::QualifiedPath { .. }
    )
}

fn optimize_expression(expr: &mut Expression) {
    match expr {
        Expression::Literal(_) | Expression::Identifier(_) | Expression::QualifiedPath { .. } => {}
        Expression::FieldAccess { base, .. }
        | Expression::Not(base)
        | Expression::Len(base)
        | Expression::FatPointer { data: base, .. } => optimize_expression(base),
        Expression::Call { func, args } => {
            optimize_expression(func);
            for arg in args {
                optimize_expression(arg);
            }
        }
        Expression::MethodCall { receiver, args, .. }
        | Expression::DynMethodCall { receiver, args, .. } => {
            optimize_expression(receiver);
            for arg in args {
                optimize_expression(arg);
            }
        }
        Expression::StructLiteral { fields, .. } | Expression::EnumLiteral { fields, .. } => {
            for (_, value) in fields {
                optimize_expression(value);
            }
        }
        Expression::BinaryOp { lhs, rhs, .. } => {
            optimize_expression(lhs);
            optimize_expression(rhs);
        }
        Expression::FormatConcat { parts } | Expression::Array { elements: parts } => {
            for part in parts {
                optimize_expression(part);
            }
        }
        Expression::Index { base, key } => {
            optimize_expression(base);
            optimize_expression(key);
        }
        Expression::If {
            condition,
            then_expr,
            else_expr,
        } => {
            optimize_expression(condition);
            optimize_expression(then_expr);
            optimize_expression(else_expr);
        }
        Expression::Closure { body, .. } => optimize_block(body),
    }
}
