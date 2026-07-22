#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod common;

use common::must_ok_parse;
use factorio_frontend::parse_module;
use factorio_ir::{expression::Expression, statement::Statement};

#[test]
fn lowers_from_impl_onto_source_into_and_impl_into_param() {
    let module = must_ok_parse(parse_module(
        r#"
        pub struct Text {
            pub caption: String,
        }

        pub enum Widget {
            Text(Text),
        }

        impl Widget {
            pub fn from_text(text: Text) -> Self {
                Self::Text(text)
            }
        }

        impl From<Text> for Widget {
            fn from(value: Text) -> Self {
                Self::from_text(value)
            }
        }

        pub struct Frame {
            pub children: Vec<Widget>,
        }

        impl Frame {
            pub fn child(mut self, child: impl Into<Widget>) -> Self {
                self.children.push(child.into());
                self
            }
        }

        pub fn build() -> impl Into<Widget> {
            Text { caption: "hi".to_string() }
        }
        "#,
        "shared.convert",
    ));

    let statements: Vec<_> = module
        .symbols
        .iter()
        .map(|symbol| &symbol.statement)
        .chain(module.body.statements.iter())
        .collect();

    let text_into = statements.iter().find_map(|statement| match statement {
        Statement::StructDecl(s) if s.name == "Text" => s.methods.iter().find(|m| m.name == "into"),
        _ => None,
    });
    assert!(text_into.is_some(), "expected Text::into from From impl");

    let frame_child = statements.iter().find_map(|statement| match statement {
        Statement::StructDecl(s) if s.name == "Frame" => {
            s.methods.iter().find(|m| m.name == "child")
        }
        _ => None,
    });
    let child = frame_child.expect("Frame::child");
    let push = child.body.statements.iter().find_map(|statement| match statement {
        Statement::Expr(Expression::MethodCall { method, args, .. }) if method == "insert" || method == "push" => {
            Some(args)
        }
        Statement::Expr(Expression::Call { func, args }) => {
            if matches!(func.as_ref(), Expression::QualifiedPath { segments } if segments.last().is_some_and(|s| s == "insert")) {
                Some(args)
            } else {
                None
            }
        }
        _ => None,
    });
    // `child.into()` should be a method call, not a transparent identity.
    let has_into_call = format!("{child:?}").contains("into");
    assert!(
        has_into_call,
        "Frame::child should call .into(); got {child:?}"
    );
    let _ = push;

    let build = statements.iter().find_map(|statement| match statement {
        Statement::FunctionDecl(f) if f.name == "build" => Some(f),
        _ => None,
    });
    let build = build.expect("build");
    let returns_into = build.body.statements.iter().any(|statement| {
        matches!(
            statement,
            Statement::Return(Some(Expression::MethodCall { method, .. })) if method == "into"
        )
    });
    assert!(
        returns_into,
        "build() -> impl Into<Widget> should wrap return with .into(); got {:?}",
        build.body.statements
    );
}
