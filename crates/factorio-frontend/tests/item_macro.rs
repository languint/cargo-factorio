#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use factorio_frontend::{FrontendError, ParseOptions, parse_module, parse_module_with_options};
use factorio_ir::{
    expression::Expression,
    lint::LintConfig,
    literal::Literal,
    locale::{LocaleEntry, LocaleFile},
    statement::Statement,
};

#[test]
fn item_macro_rewrites_relative_icon_and_emits_items_const() {
    let source = r#"
        item! {
            widget {
                name = "my-mod-widget",
                icon = "graphics/icon.png",
                stack_size = 50,
                icon_size = 64,
            }
        }
    "#;

    let lints = LintConfig::allow_all();
    let mut diagnostics = Vec::new();
    let module = parse_module_with_options(
        source,
        "data",
        &ParseOptions::new(&lints).with_mod_name("my_mod"),
        &mut diagnostics,
    )
    .expect("parse");

    let items = module
        .symbols
        .iter()
        .find_map(|symbol| match &symbol.statement {
            Statement::StructDecl(s) if s.name == "Items" => Some(s),
            _ => None,
        })
        .expect("Items struct");
    assert!(
        items.constants.iter().any(|(name, value)| {
            name == "WIDGET"
                && matches!(
                    value,
                    Expression::Literal(Literal::String(s)) if s == "my-mod-widget"
                )
        }),
        "expected Items::WIDGET const, got {:?}",
        items.constants
    );

    let register = module
        .symbols
        .iter()
        .find_map(|symbol| match &symbol.statement {
            Statement::FunctionDecl(f) if f.name == "register" => Some(f),
            _ => None,
        })
        .expect("register");
    let body = format!("{:?}", register.body);
    assert!(
        body.contains("__my_mod__/graphics/icon.png"),
        "expected rewritten icon path in register body: {body}"
    );
}

#[test]
fn item_macro_rejects_relative_icon_without_mod_name() {
    let source = r#"
        item! {
            widget {
                name = "my-mod-widget",
                icon = "graphics/icon.png",
                stack_size = 50,
            }
        }
    "#;

    let err = parse_module(source, "data").expect_err("should fail without mod_name");
    assert!(
        matches!(err, FrontendError::ItemIconNeedsModName { .. }),
        "unexpected error: {err:?}"
    );
}

#[test]
fn item_macro_locale_resolves_items_const_keys() {
    let source = r#"
        item! {
            widget {
                name = "my-mod-widget",
                icon = "__my_mod__/graphics/icon.png",
                stack_size = 50,
            }
        }

        locale! {
            file = "items",
            en {
                item_name {
                    Items::WIDGET = "Widget",
                }
                item_description {
                    Items::WIDGET = "A sample item.",
                }
            }
        }
    "#;

    let module = parse_module(source, "data").expect("parse");
    assert_eq!(
        module.locales,
        vec![LocaleFile {
            lang: "en".to_string(),
            file: "items".to_string(),
            entries: vec![
                LocaleEntry {
                    category: Some("item-name".to_string()),
                    key: "my-mod-widget".to_string(),
                    value: "Widget".to_string(),
                },
                LocaleEntry {
                    category: Some("item-description".to_string()),
                    key: "my-mod-widget".to_string(),
                    value: "A sample item.".to_string(),
                },
            ],
        }]
    );
}
