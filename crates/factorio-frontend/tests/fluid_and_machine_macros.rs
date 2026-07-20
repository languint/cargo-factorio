#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use factorio_frontend::{ParseOptions, parse_module_with_options};
use factorio_ir::{
    expression::Expression, lint::LintConfig, literal::Literal, statement::Statement,
};

#[test]
fn fluid_macro_emits_fluids_const_and_register() {
    let source = r#"
        fluid! {
            steam_mix {
                name = "my-mod-steam-mix",
                icon = "graphics/fluid.png",
                default_temperature = 15.0,
                base_color = { r = 0.5, g = 0.5, b = 0.5 },
                flow_color = { r = 0.7, g = 0.7, b = 0.7 },
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

    let fluids = module
        .symbols
        .iter()
        .find_map(|symbol| match &symbol.statement {
            Statement::StructDecl(s) if s.name == "Fluids" => Some(s),
            _ => None,
        })
        .expect("Fluids struct");
    assert!(
        fluids.constants.iter().any(|(name, value)| {
            name == "STEAM_MIX"
                && matches!(
                    value,
                    Expression::Literal(Literal::String(s)) if s == "my-mod-steam-mix"
                )
        }),
        "expected Fluids::STEAM_MIX, got {:?}",
        fluids.constants
    );

    let register = module
        .symbols
        .iter()
        .find_map(|symbol| match &symbol.statement {
            Statement::FunctionDecl(f) if f.name == "register_fluids" => Some(f),
            _ => None,
        })
        .expect("register_fluids");
    let body = format!("{:?}", register.body);
    assert!(
        body.contains("__my_mod__/graphics/fluid.png"),
        "expected rewritten icon: {body}"
    );
    assert!(
        body.contains("Color") && body.contains("default_temperature"),
        "expected fluid fields: {body}"
    );
}

#[test]
fn assembling_machine_macro_emits_register() {
    let source = r#"
        assembling_machine! {
            widget_assembler {
                name = "my-mod-assembler",
                icon = "graphics/entity.png",
                crafting_speed = 0.5,
                crafting_categories = ["crafting"],
                energy_usage = "150kW",
                energy_type = "electric",
                usage_priority = "secondary-input",
                flags = ["placeable-neutral", "player-creation"],
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

    assert!(
        module.symbols.iter().any(|symbol| {
            matches!(
                &symbol.statement,
                Statement::StructDecl(s) if s.name == "AssemblingMachines"
            )
        }),
        "expected AssemblingMachines"
    );
    let register = module
        .symbols
        .iter()
        .find_map(|symbol| match &symbol.statement {
            Statement::FunctionDecl(f) if f.name == "register_assembling_machines" => Some(f),
            _ => None,
        })
        .expect("register_assembling_machines");
    let body = format!("{:?}", register.body);
    assert!(
        body.contains("EnergySource") && body.contains("crafting"),
        "expected machine fields: {body}"
    );
}
