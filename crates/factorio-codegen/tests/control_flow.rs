mod common;

use common::must_ok;
use factorio_codegen::LuaGenerator;
use factorio_ir::{
    block::Block,
    expression::Expression,
    function::{Function, Parameter},
    literal::Literal,
    module::{Module, Symbol},
    operator::Operator,
    scope::Scope,
    stage::Stage,
    statement::Statement,
    r#type::Type,
};

#[test]
fn generates_while_continue_and_break() {
    let module = Module {
        name: "loops".to_string(),
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
                name: "tick".to_string(),
                params: vec![Parameter {
                    name: "n".to_string(),
                    r#type: Type::Int,
                    source_type: None,
                }],
                body: Block {
                    statements: vec![Statement::While {
                        condition: Expression::BinaryOp {
                            lhs: Box::new(Expression::Identifier("n".to_string())),
                            op: Operator::Gt,
                            rhs: Box::new(Expression::Literal(Literal::Int(0))),
                        },
                        body: vec![
                            Statement::Conditional {
                                condition: Expression::BinaryOp {
                                    lhs: Box::new(Expression::Identifier("n".to_string())),
                                    op: Operator::Eq,
                                    rhs: Box::new(Expression::Literal(Literal::Int(1))),
                                },
                                then_block: vec![Statement::Break],
                                else_block: vec![],
                            },
                            Statement::Continue,
                        ],
                    }],
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

    let output = must_ok(LuaGenerator::new().generate_module(&module));
    assert!(output.contains("while n > 0 do"));
    assert!(output.contains("break"));
    assert!(output.contains("goto __continue_1"));
    assert!(output.contains("::__continue_1::"));
}

#[test]
fn nested_else_if_emits_elseif() {
    let module = Module {
        name: "m".to_string(),
        stage: Stage::Control,
        body: Block {
            statements: vec![Statement::Conditional {
                condition: Expression::Identifier("a".to_string()),
                then_block: vec![Statement::Return(Some(Expression::Literal(Literal::Int(
                    1,
                ))))],
                else_block: vec![Statement::Conditional {
                    condition: Expression::Identifier("b".to_string()),
                    then_block: vec![Statement::Return(Some(Expression::Literal(Literal::Int(
                        2,
                    ))))],
                    else_block: vec![Statement::Return(Some(Expression::Literal(Literal::Int(
                        3,
                    ))))],
                }],
            }],
        },
        imports: vec![],
        submodules: vec![],
        locales: vec![],
        pending_locales: vec![],
        vtables: vec![],
        symbols: vec![],
    };

    let output = must_ok(LuaGenerator::new().generate_module(&module));
    assert!(
        output.contains("elseif b then"),
        "expected elseif chain, got:\n{output}"
    );
    assert!(
        !output.contains("else\n  if b then"),
        "should not nest else/if:\n{output}"
    );
    let end_count = output.matches("\nend").count() + usize::from(output.ends_with("end"));
    assert!(
        end_count <= 2,
        "elseif chain should use a single end for the if, got {end_count} in:\n{output}"
    );
}
