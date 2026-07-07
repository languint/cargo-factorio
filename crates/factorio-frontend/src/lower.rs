use syn::spanned::Spanned;
use syn::{
    BinOp, Block, Expr, ExprBinary, ExprLit, ExprPath, File, Item, ItemFn, Lit, PatType, Signature,
    Stmt, Type, Visibility,
};

use crate::error::{FrontendError, FrontendResult};

/// Parse Rust source into a [`factorio_ir::module::Module`].
///
/// `module_name` is used as the module identifier in the resulting IR.
pub fn parse_module(
    source: &str,
    module_name: &str,
) -> FrontendResult<factorio_ir::module::Module> {
    let file = syn::parse_file(source)?;
    lower_module(&file, module_name)
}

/// Lower a parsed Rust file into a module.
fn lower_module(file: &File, module_name: &str) -> FrontendResult<factorio_ir::module::Module> {
    let mut body = Vec::new();
    let mut symbols = Vec::new();

    for item in &file.items {
        match item {
            Item::Fn(function) => {
                let lowered =
                    factorio_ir::statement::Statement::FunctionDecl(lower_function(function)?);
                match &function.vis {
                    Visibility::Public(_) => symbols.push(factorio_ir::module::Symbol {
                        scope: factorio_ir::scope::Scope::Public,
                        statement: lowered,
                    }),
                    _ => body.push(lowered),
                }
            }
            item => {
                return Err(FrontendError::UnsupportedItem {
                    item: item_name(item),
                    location: location(item),
                });
            }
        }
    }

    Ok(factorio_ir::module::Module {
        name: module_name.to_string(),
        body: factorio_ir::block::Block { statements: body },
        symbols,
    })
}

fn lower_function(function: &ItemFn) -> FrontendResult<factorio_ir::function::Function> {
    Ok(factorio_ir::function::Function {
        name: function.sig.ident.to_string(),
        params: lower_parameters(&function.sig)?,
        body: lower_block(&function.block)?,
    })
}

fn lower_parameters(
    signature: &Signature,
) -> FrontendResult<Vec<factorio_ir::function::Parameter>> {
    signature
        .inputs
        .iter()
        .map(lower_parameter)
        .collect::<FrontendResult<Vec<_>>>()
}

fn lower_parameter(input: &syn::FnArg) -> FrontendResult<factorio_ir::function::Parameter> {
    match input {
        syn::FnArg::Receiver(_receiver) => Ok(factorio_ir::function::Parameter {
            // `&self` and `&mut self` become a `self` parameter.
            name: "self".to_string(),
            r#type: factorio_ir::r#type::Type::Void,
        }),
        syn::FnArg::Typed(PatType { pat, ty, .. }) => {
            let name = lower_binding_pattern(pat)?;
            let r#type = lower_type(ty)?;

            Ok(factorio_ir::function::Parameter { name, r#type })
        }
    }
}

/// Lower a block of Rust statements into IR statements.
fn lower_block(block: &Block) -> FrontendResult<factorio_ir::block::Block> {
    let mut statements = Vec::new();
    let last_index = block.stmts.len().saturating_sub(1);

    for (index, statement) in block.stmts.iter().enumerate() {
        let is_tail = index == last_index;
        statements.extend(lower_statement(statement, is_tail)?);
    }

    Ok(factorio_ir::block::Block { statements })
}

/// Lower a single Rust statement into zero or more IR statements.
fn lower_statement(
    statement: &Stmt,
    is_tail: bool,
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
            let value = lower_expression(&init.expr)?;
            let ty = match annotated_type {
                Some(ty) => ty,
                None => infer_type_from_expression(&value).ok_or_else(|| {
                    FrontendError::UnsupportedType {
                        ty: "missing type annotation".to_string(),
                        location: location(local),
                    }
                })?,
            };

            Ok(vec![factorio_ir::statement::Statement::VariableDecl {
                name,
                ty,
                value,
            }])
        }
        Stmt::Item(syn::Item::Fn(function)) => {
            Ok(vec![factorio_ir::statement::Statement::FunctionDecl(
                lower_function(function)?,
            )])
        }
        Stmt::Item(item) => Err(FrontendError::UnsupportedItem {
            item: item_name(item),
            location: location(item),
        }),
        Stmt::Expr(expression, semi) => {
            lower_expression_statement(expression, semi.is_some(), is_tail)
        }
        _ => Err(FrontendError::UnsupportedStatement {
            location: location(statement),
        }),
    }
}

fn lower_expression_statement(
    expression: &Expr,
    has_semi: bool,
    is_tail: bool,
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    if has_semi {
        return Ok(vec![lower_semicolon_expression(expression)?]);
    }

    if is_tail {
        return Ok(vec![lower_tail_expression(expression)?]);
    }

    Err(FrontendError::UnsupportedStatement {
        location: location(expression),
    })
}

/// Lower a tail expression without a trailing semicolon.
///
/// Control-flow expressions such as `if` remain statements. Other expressions
/// become implicit `return` values.
fn lower_tail_expression(expression: &Expr) -> FrontendResult<factorio_ir::statement::Statement> {
    match expression {
        Expr::If(if_expression) => lower_if_expression(if_expression),
        Expr::Return(return_expression) => Ok(factorio_ir::statement::Statement::Return(
            match return_expression.expr.as_deref() {
                Some(value) => Some(lower_expression(value)?),
                None => None,
            },
        )),
        _ => Ok(factorio_ir::statement::Statement::Return(Some(
            lower_expression(expression)?,
        ))),
    }
}

fn lower_semicolon_expression(
    expression: &Expr,
) -> FrontendResult<factorio_ir::statement::Statement> {
    match expression {
        Expr::Return(return_expression) => Ok(factorio_ir::statement::Statement::Return(
            match return_expression.expr.as_deref() {
                Some(value) => Some(lower_expression(value)?),
                None => None,
            },
        )),
        Expr::Assign(assign) => Ok(factorio_ir::statement::Statement::Assignment {
            target: lower_assignment_target(&assign.left)?,
            value: lower_expression(&assign.right)?,
        }),
        Expr::If(if_expression) => lower_if_expression(if_expression),
        _ => Err(FrontendError::UnsupportedStatement {
            location: location(expression),
        }),
    }
}

fn lower_if_expression(
    if_expression: &syn::ExprIf,
) -> FrontendResult<factorio_ir::statement::Statement> {
    let condition = lower_expression(&if_expression.cond)?;
    let then_block = lower_block_statements(&if_expression.then_branch.stmts)?;
    let else_block = match &if_expression.else_branch {
        Some((_, else_branch)) => lower_branch_statements(else_branch)?,
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
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    match expression {
        Expr::Block(block) => lower_block_statements(&block.block.stmts),
        _ => Err(FrontendError::UnsupportedStatement {
            location: location(expression),
        }),
    }
}

fn lower_block_statements(
    statements: &[Stmt],
) -> FrontendResult<Vec<factorio_ir::statement::Statement>> {
    let mut lowered = Vec::new();
    let last_index = statements.len().saturating_sub(1);

    for (index, statement) in statements.iter().enumerate() {
        let is_tail = index == last_index;
        lowered.extend(lower_statement(statement, is_tail)?);
    }

    Ok(lowered)
}

/// Lower a Rust expression into IR.
fn lower_expression(expression: &Expr) -> FrontendResult<factorio_ir::expression::Expression> {
    match expression {
        Expr::Binary(binary) => lower_binary_expression(binary),
        Expr::Lit(literal) => lower_literal_expression(literal),
        Expr::Path(path) => lower_path_expression(path),
        _ => Err(FrontendError::UnsupportedExpression {
            location: location(expression),
        }),
    }
}

fn lower_binary_expression(
    binary: &ExprBinary,
) -> FrontendResult<factorio_ir::expression::Expression> {
    let lhs = lower_expression(&binary.left)?;
    let op = lower_binary_operator(&binary.op)?;
    let rhs = lower_expression(&binary.right)?;

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
        _ => {
            return Err(FrontendError::UnsupportedExpression {
                location: location(literal),
            });
        }
    };

    Ok(factorio_ir::expression::Expression::Literal(literal))
}

fn lower_path_expression(path: &ExprPath) -> FrontendResult<factorio_ir::expression::Expression> {
    let ident = single_path_segment(path).ok_or_else(|| FrontendError::UnsupportedExpression {
        location: location(path),
    })?;

    Ok(factorio_ir::expression::Expression::Identifier(
        ident.to_string(),
    ))
}

fn single_path_segment(path: &ExprPath) -> Option<&syn::Ident> {
    match path.path.segments.len() {
        1 => Some(&path.path.segments.first()?.ident),
        _ => None,
    }
}

fn lower_assignment_target(
    expression: &Expr,
) -> FrontendResult<factorio_ir::expression::Expression> {
    match expression {
        Expr::Path(path) => lower_path_expression(path),
        _ => Err(FrontendError::ExpectedIdentifierAssignmentTarget {
            location: location(expression),
        }),
    }
}

/// Infer a type from a literal expression when a `let` binding has no annotation.
fn infer_type_from_expression(
    expression: &factorio_ir::expression::Expression,
) -> Option<factorio_ir::r#type::Type> {
    match expression {
        factorio_ir::expression::Expression::Literal(literal) => match literal {
            factorio_ir::literal::Literal::Int(_) => Some(factorio_ir::r#type::Type::Int),
            factorio_ir::literal::Literal::Float(_) => Some(factorio_ir::r#type::Type::Float),
            factorio_ir::literal::Literal::String(_) => Some(factorio_ir::r#type::Type::Str),
        },
        _ => None,
    }
}

/// Lower a Rust type into IR.
fn lower_type(ty: &Type) -> FrontendResult<factorio_ir::r#type::Type> {
    match ty {
        Type::Path(path) => lower_path_type(path),
        Type::Tuple(tuple) if tuple.elems.is_empty() => Ok(factorio_ir::r#type::Type::Void),
        Type::Reference(reference) if is_self_type(&reference.elem) => {
            Ok(factorio_ir::r#type::Type::Void)
        }
        _ => Err(FrontendError::UnsupportedType {
            ty: "unsupported type".to_string(),
            location: location(ty),
        }),
    }
}

fn lower_path_type(path: &syn::TypePath) -> FrontendResult<factorio_ir::r#type::Type> {
    let segment = path
        .path
        .segments
        .last()
        .ok_or_else(|| FrontendError::UnsupportedType {
            ty: "empty path".to_string(),
            location: location(path),
        })?;

    let ty = match segment.ident.to_string().as_str() {
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" => factorio_ir::r#type::Type::Int,
        "f32" | "f64" => factorio_ir::r#type::Type::Float,
        "str" | "String" => factorio_ir::r#type::Type::Str,
        _ => {
            return Err(FrontendError::UnsupportedType {
                ty: segment.ident.to_string(),
                location: location(path),
            });
        }
    };

    Ok(ty)
}

fn is_self_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.qself.is_none() && path.path.is_ident("Self"))
}

fn lower_binding(
    pattern: &syn::Pat,
) -> FrontendResult<(String, Option<factorio_ir::r#type::Type>)> {
    match pattern {
        syn::Pat::Type(pat_type) => {
            let name = lower_binding_pattern(&pat_type.pat)?;
            let ty = lower_type(&pat_type.ty)?;
            Ok((name, Some(ty)))
        }
        pattern => {
            let name = lower_binding_pattern(pattern)?;
            Ok((name, None))
        }
    }
}

fn lower_binding_pattern(pattern: &syn::Pat) -> FrontendResult<String> {
    match pattern {
        syn::Pat::Ident(ident) => Ok(ident.ident.to_string()),
        syn::Pat::Type(pat_type) => lower_binding_pattern(&pat_type.pat),
        _ => Err(FrontendError::ExpectedIdentifierPattern {
            location: location(pattern),
        }),
    }
}

/// Returns a source location string for error reporting.
fn location(span: impl Spanned) -> String {
    format!("{:?}", span.span())
}

/// Returns a short description of a top-level item for error reporting.
fn item_name(item: &syn::Item) -> String {
    match item {
        syn::Item::Fn(function) => format!("fn {}", function.sig.ident),
        syn::Item::Mod(module) => format!("mod {}", module.ident),
        syn::Item::Struct(item) => format!("struct {}", item.ident),
        syn::Item::Enum(item) => format!("enum {}", item.ident),
        syn::Item::Const(item) => format!("const {}", item.ident),
        syn::Item::Static(item) => format!("static {}", item.ident),
        syn::Item::Use(_) => "use".to_string(),
        syn::Item::Type(item) => format!("type {}", item.ident),
        syn::Item::Macro(_) => "macro".to_string(),
        _ => "item".to_string(),
    }
}
