mod common;

use common::{must_ok, must_ok_parse};
use factorio_codegen::LuaGenerator;
use factorio_frontend::parse_module;
use factorio_ir::module::{ImportedItem, ModuleImport};

#[test]
fn parses_crate_item_use() {
    let source = r"
use crate::player::MyPlayer;

pub fn on_init() {
    let player = MyPlayer::new();
}
";

    let module = must_ok_parse(parse_module(source, "on_init"));

    assert_eq!(
        module.imports,
        vec![ModuleImport {
            module: "player".to_string(),
            local: "player".to_string(),
            items: vec![ImportedItem {
                name: "MyPlayer".to_string(),
                local: "MyPlayer".to_string(),
            }],
        }]
    );
}

#[test]
fn parses_grouped_use() {
    let source = r"
use crate::player::{MyPlayer, OtherThing};

pub fn on_init() {}
";

    let module = must_ok_parse(parse_module(source, "on_init"));

    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].module, "player");
    assert_eq!(module.imports[0].items.len(), 2);
}

#[test]
fn ignores_external_crate_use() {
    let source = r"
use factorio::event::OnInit;
use crate::player::MyPlayer;

pub fn on_init() {}
";

    let module = must_ok_parse(parse_module(source, "on_init"));

    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].module, "player");
}

#[test]
fn generates_require_for_imports() {
    let source = r"
use crate::player::MyPlayer;

pub fn on_init() {
    let player = MyPlayer::new();
}
";

    let module = must_ok_parse(parse_module(source, "on_init"));
    let lua = must_ok(LuaGenerator::new().generate_module(&module));

    assert!(lua.contains("local player = require(\"player\")"));
    assert!(lua.contains("local MyPlayer = player.MyPlayer"));
    assert!(lua.contains("local player = MyPlayer.new()"));
}

#[test]
fn inline_crate_path_generates_require() {
    let source = r"
pub fn on_init() {
    let player = crate::player::MyPlayer::new();
}
";

    let module = must_ok_parse(parse_module(source, "on_init"));
    let lua = must_ok(LuaGenerator::new().generate_module(&module));

    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].module, "player");
    assert!(lua.contains("local player = require(\"player\")"));
    assert!(lua.contains("local player = player.MyPlayer.new()"));
}
