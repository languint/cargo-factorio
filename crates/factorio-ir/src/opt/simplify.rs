use crate::{
    block::Block, expression::Expression, function::Function, literal::Literal, module::Module,
    operator::Operator, statement::Statement, r#type::Type,
};

pub(super) fn optimize_module(module: &mut Module) {
    optimize_block(&mut module.body);
    for symbol in &mut module.symbols {
        optimize_statement(&mut symbol.statement);
    }
}

fn optimize_block(block: &mut Block) {
    let statements = std::mem::take(&mut block.statements);
    block.statements = simplify_statements(&statements);
}

fn optimize_statement(statement: &mut Statement) {
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
        }
        Statement::Conditional {
            then_block,
            else_block,
            ..
        } => {
            let then_taken = std::mem::take(then_block);
            *then_block = simplify_statements(&then_taken);
            let else_taken = std::mem::take(else_block);
            *else_block = simplify_statements(&else_taken);
        }
        Statement::ForIn { body, .. }
        | Statement::ForNumeric { body, .. }
        | Statement::While { body, .. } => {
            let taken = std::mem::take(body);
            *body = simplify_statements(&taken);
        }
        Statement::VariableDecl { .. }
        | Statement::Assignment { .. }
        | Statement::Return(_)
        | Statement::Expr(_)
        | Statement::Continue
        | Statement::Break => {}
    }
}

fn optimize_function(function: &mut Function) {
    optimize_block(&mut function.body);
}

fn simplify_statements(statements: &[Statement]) -> Vec<Statement> {
    let mut out = Vec::with_capacity(statements.len());
    let mut i = 0;
    while i < statements.len() {
        if let Some((consumed, replacement)) = try_simplify_unwrap_or(&statements[i..]) {
            out.extend(replacement);
            i += consumed;
            continue;
        }
        let mut statement = statements[i].clone();
        optimize_statement(&mut statement);
        out.push(statement);
        i += 1;
    }
    out
}

/// `local n = nil; [local tmp = recv;] if tmp ~= nil then n = tmp else n = d end`
/// -> `local n = recv; if n == nil then n = d end`
fn try_simplify_unwrap_or(stmts: &[Statement]) -> Option<(usize, Vec<Statement>)> {
    let Statement::VariableDecl {
        name: dest,
        ty,
        source_type,
        value: Expression::Literal(Literal::Nil),
    } = &stmts[0]
    else {
        return None;
    };

    if stmts.len() >= 3
        && let Statement::VariableDecl {
            name: tmp,
            value: recv,
            ..
        } = &stmts[1]
        && let Statement::Conditional {
            condition,
            then_block,
            else_block,
        } = &stmts[2]
        && is_ne_nil(condition, tmp)
        && is_single_assign_ident(then_block, dest, tmp)
        && let Some(default) = single_assign_value(else_block, dest)
    {
        return Some((
            3,
            rewrite_unwrap_or(dest, ty, source_type.as_ref(), recv.clone(), default),
        ));
    }

    if stmts.len() >= 2
        && let Statement::Conditional {
            condition,
            then_block,
            else_block,
        } = &stmts[1]
        && let Some(src) = ne_nil_ident(condition)
        && is_single_assign_ident(then_block, dest, &src)
        && let Some(default) = single_assign_value(else_block, dest)
    {
        return Some((
            2,
            rewrite_unwrap_or(
                dest,
                ty,
                source_type.as_ref(),
                Expression::Identifier(src),
                default,
            ),
        ));
    }

    None
}

fn rewrite_unwrap_or(
    dest: &str,
    ty: &Type,
    source_type: Option<&String>,
    value: Expression,
    default: Expression,
) -> Vec<Statement> {
    vec![
        Statement::VariableDecl {
            name: dest.to_string(),
            ty: ty.clone(),
            source_type: source_type.cloned(),
            value,
        },
        Statement::Conditional {
            condition: Expression::BinaryOp {
                lhs: Box::new(Expression::Identifier(dest.to_string())),
                op: Operator::Eq,
                rhs: Box::new(Expression::Literal(Literal::Nil)),
            },
            then_block: vec![Statement::Assignment {
                target: Expression::Identifier(dest.to_string()),
                value: default,
            }],
            else_block: vec![],
        },
    ]
}

fn is_ne_nil(condition: &Expression, name: &str) -> bool {
    matches!(
        condition,
        Expression::BinaryOp {
            lhs,
            op: Operator::Ne,
            rhs,
        } if matches!(lhs.as_ref(), Expression::Identifier(id) if id == name)
            && matches!(rhs.as_ref(), Expression::Literal(Literal::Nil))
    )
}

fn ne_nil_ident(condition: &Expression) -> Option<String> {
    match condition {
        Expression::BinaryOp {
            lhs,
            op: Operator::Ne,
            rhs,
        } if matches!(rhs.as_ref(), Expression::Literal(Literal::Nil)) => match lhs.as_ref() {
            Expression::Identifier(id) => Some(id.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn is_single_assign_ident(block: &[Statement], dest: &str, src: &str) -> bool {
    matches!(
        block,
        [Statement::Assignment {
            target: Expression::Identifier(t),
            value: Expression::Identifier(v),
        }] if t == dest && v == src
    )
}

fn single_assign_value(block: &[Statement], dest: &str) -> Option<Expression> {
    match block {
        [
            Statement::Assignment {
                target: Expression::Identifier(t),
                value,
            },
        ] if t == dest => Some(value.clone()),
        _ => None,
    }
}
