#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use factorio_frontend::{ParseOptions, parse_module_with_options};
use factorio_ir::{
    expression::Expression, lint::LintConfig, literal::Literal, statement::Statement,
};

#[test]
fn technology_macro_emits_technologies_const_and_register_technologies() {
    let source = r#"
        technology! {
            widget_tech {
                name = "my-mod-widget",
                icon = "graphics/technology.png",
                icon_size = 256,
                prerequisites = ["automation"],
                unlock_recipes = ["my-mod-widget"],
                unit_count = 50,
                unit_time = 30.0,
                unit_ingredients = [
                    { name = "automation-science-pack", amount = 1 },
                ],
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

    let technologies = module
        .symbols
        .iter()
        .find_map(|symbol| match &symbol.statement {
            Statement::StructDecl(s) if s.name == "Technologies" => Some(s),
            _ => None,
        })
        .expect("Technologies struct");
    assert!(
        technologies.constants.iter().any(|(name, value)| {
            name == "WIDGET_TECH"
                && matches!(
                    value,
                    Expression::Literal(Literal::String(s)) if s == "my-mod-widget"
                )
        }),
        "expected Technologies::WIDGET_TECH const, got {:?}",
        technologies.constants
    );

    let register = module
        .symbols
        .iter()
        .find_map(|symbol| match &symbol.statement {
            Statement::FunctionDecl(f) if f.name == "register_technologies" => Some(f),
            _ => None,
        })
        .expect("register_technologies");
    let body = format!("{:?}", register.body);
    assert!(
        body.contains("__my_mod__/graphics/technology.png"),
        "expected rewritten icon path in register_technologies body: {body}"
    );
    assert!(
        body.contains("UnlockRecipeEffect") && body.contains("my-mod-widget"),
        "expected unlock recipe effect in register_technologies body: {body}"
    );
    assert!(
        body.contains("TechnologyUnitIngredient")
            && body.contains("automation-science-pack"),
        "expected unit ingredients in register_technologies body: {body}"
    );
    assert!(
        body.contains("automation"),
        "expected prerequisite in register_technologies body: {body}"
    );
}
