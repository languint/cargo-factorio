#![allow(clippy::expect_used, clippy::panic)]

mod common;

use common::must_ok_parse;
use factorio_frontend::parse_module;
use factorio_ir::statement::Statement;

#[test]
fn shared_inline_marks_export() {
    let source = r"
#[factorio_rs::inline]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
";
    let module = must_ok_parse(parse_module(source, "shared.api"));
    let Statement::FunctionDecl(function) = &module.symbols[0].statement else {
        panic!("expected function");
    };
    assert!(function.inline);
    assert!(function.export.is_some());
}

#[test]
fn inline_rejected_outside_shared() {
    let source = r"
#[factorio_rs::inline]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
";
    let err = parse_module(source, "control").expect_err("inline in control");
    let msg = err.to_string();
    assert!(msg.contains("inline") || msg.contains("shared"), "{msg}");
}
