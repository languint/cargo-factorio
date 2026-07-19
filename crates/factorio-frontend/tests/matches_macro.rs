#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod common;

use common::must_ok_parse;
use factorio_frontend::parse_module;
use factorio_ir::{expression::Expression, statement::Statement};

#[test]
fn lowers_matches_macro_for_enum_and_option() {
    let module = must_ok_parse(parse_module(
        r"
        pub enum Msg {
            Quit,
            Move(i64),
        }

        impl Msg {
            pub fn is_quit(&self) -> bool {
                matches!(self, Self::Quit)
            }
        }

        pub fn check(msg: Msg, opt: Option<i64>) -> bool {
            let a = matches!(msg, Msg::Move(_));
            let b = matches!(opt, Some(n) if n > 0);
            let c = matches!(opt, None | Some(0));
            a && b && c
        }
        ",
        "shared.matches",
    ));

    let check = module
        .symbols
        .iter()
        .find_map(|symbol| match &symbol.statement {
            Statement::FunctionDecl(function) if function.name == "check" => Some(function),
            _ => None,
        })
        .expect("expected check function");

    assert!(check.body.statements.len() >= 3);
    for statement in check.body.statements.iter().take(3) {
        let Statement::VariableDecl { value, .. } = statement else {
            panic!("expected matches! binding, got {statement:?}");
        };
        assert!(
            matches!(
                value,
                Expression::Call { func, args }
                    if args.is_empty() && matches!(func.as_ref(), Expression::Closure { .. })
            ),
            "matches! should desugar to match IIFE, got {value:?}"
        );
    }
}

#[test]
fn rejects_malformed_matches_macro() {
    assert!(parse_module("pub fn f(x: i64) -> bool { matches!(x) }", "control").is_err());
}
