#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::panic_in_result_fn
)]

mod common;

use common::{assert_lua_parses, must_ok};
use factorio_codegen::LuaGenerator;
use factorio_ir::{
    block::Block,
    expression::Expression,
    function::Function,
    literal::Literal,
    module::{Module, Symbol},
    opt::optimize_modules,
    scope::Scope,
    stage::Stage,
    statement::Statement,
    r#type::Type,
};

#[test]
fn optimized_if_expr_let_emits_statement_if() {
    let mut module = Module {
        name: "control".to_string(),
        stage: Stage::Control,
        body: Block { statements: vec![] },
        imports: vec![],
        submodules: vec![],
        locales: vec![],
        pending_locales: vec![],
        vtables: vec![],
        symbols: vec![Symbol {
            scope: Scope::Public,
            statement: Statement::FunctionDecl(Function {
                name: "pick".to_string(),
                params: vec![],
                body: Block {
                    statements: vec![
                        Statement::VariableDecl {
                            name: "x".to_string(),
                            ty: Type::Int,
                            source_type: None,
                            value: Expression::If {
                                condition: Box::new(Expression::Identifier("c".to_string())),
                                then_expr: Box::new(Expression::Literal(Literal::Int(1))),
                                else_expr: Box::new(Expression::Literal(Literal::Int(0))),
                            },
                        },
                        Statement::Return(Some(Expression::Identifier("x".to_string()))),
                    ],
                },
                doc: None,
                debug: None,
                event: None,
                event_filter: None,
                export: None,
                inline: false,
            }),
        }],
    };

    optimize_modules(std::slice::from_mut(&mut module));
    let lua = must_ok(LuaGenerator::new().generate_module(&module));
    assert!(
        !lua.contains("(function()"),
        "statement-context if should not be an IIFE:\n{lua}"
    );
    assert!(lua.contains("if c then"), "{lua}");
    assert!(lua.contains("x = 1"), "{lua}");
    assert!(lua.contains("x = 0"), "{lua}");
    assert_lua_parses(&lua);
}

#[test]
fn expression_position_if_still_uses_iife() {
    let expr = Expression::If {
        condition: Box::new(Expression::Identifier("c".to_string())),
        then_expr: Box::new(Expression::Literal(Literal::Bool(false))),
        else_expr: Box::new(Expression::Literal(Literal::Bool(true))),
    };
    let lua = LuaGenerator::new().generate_expression(&expr);
    assert!(
        lua.contains("(function()"),
        "mid-expression if must stay an IIFE for falsey safety: {lua}"
    );
}
