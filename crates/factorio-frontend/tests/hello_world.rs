use std::path::PathBuf;

use factorio_codegen::LuaGenerator;
use factorio_frontend::parse_module;
use factorio_ir::{expression::Expression, statement::Statement};

#[test]
fn parses_hello_world_sources() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/hello_world/src");
    let on_init = std::fs::read_to_string(root.join("on_init.rs")).unwrap();
    let player = std::fs::read_to_string(root.join("player.rs")).unwrap();

    let player_module = parse_module(&player, "player").unwrap_or_else(|error| {
        panic!("player.rs failed: {error}");
    });
    let on_init_module = parse_module(&on_init, "on_init").unwrap_or_else(|error| {
        panic!("on_init.rs failed: {error}");
    });

    let Statement::StructDecl(player_struct) = &player_module.symbols[0].statement else {
        panic!("expected struct");
    };
    let new_method = player_struct
        .methods
        .iter()
        .find(|method| method.name == "new")
        .expect("new method");
    let Statement::Return(Some(Expression::StructLiteral { fields })) =
        &new_method.body.statements[0]
    else {
        panic!("expected struct literal return");
    };
    assert_eq!(
        fields[0].1,
        Expression::QualifiedPath {
            segments: vec!["MyPlayer".to_string(), "DEFAULT_HEALTH".to_string()],
        }
    );

    let lua = LuaGenerator::new()
        .generate_module(&player_module)
        .expect("generate player lua");
    assert!(lua.contains("return { health = player.MyPlayer.DEFAULT_HEALTH }"));

    drop(on_init_module);
}
