#![allow(
    clippy::expect_used,
    clippy::literal_string_with_formatting_args,
    clippy::needless_raw_string_hashes,
    clippy::panic,
    clippy::unwrap_used
)]
mod common;

use std::collections::{BTreeMap, BTreeSet};

use common::{must_ok, must_ok_parse};
use factorio_codegen::LuaGenerator;
use factorio_frontend::{FactorioBinding, ParseOptions, parse_module, parse_module_with_options};
use factorio_ir::{
    lint::LintConfig,
    module::{ImportedItem, ModuleImport},
};

#[test]
fn parses_crate_item_use() {
    let source = r"
use crate::shared::player::MyPlayer;

pub fn on_init() {
    let player = MyPlayer::new();
}
";

    let module = must_ok_parse(parse_module(source, "control.on_init"));

    assert_eq!(
        module.imports,
        vec![ModuleImport {
            module: "shared.player".to_string(),
            local: "shared_player".to_string(),
            items: vec![ImportedItem {
                name: "MyPlayer".to_string(),
                local: "MyPlayer".to_string(),
            }],
            factorio_mod: None,
            module_root: None,
        }]
    );
}

#[test]
fn parses_grouped_use() {
    let source = r"
use crate::shared::player::{MyPlayer, OtherThing};

pub fn on_init() {}
";

    let module = must_ok_parse(parse_module(source, "control.on_init"));

    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].module, "shared.player");
    assert_eq!(module.imports[0].local, "shared_player");
    assert_eq!(module.imports[0].items.len(), 2);
}

#[test]
fn ignores_external_crate_use() {
    let source = r"
use factorio_rs::prelude::OnSingleplayerInit;
use crate::shared::player::MyPlayer;

pub fn on_init() {}
";

    let module = must_ok_parse(parse_module(source, "control.on_init"));

    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].module, "shared.player");
}

#[test]
fn ignores_external_glob_use() {
    let source = r"
use factorio_rs::prelude::*;
use crate::shared::player::MyPlayer;

pub fn on_init() {}
";

    let module = must_ok_parse(parse_module(source, "control.on_init"));

    // External glob is ignored; only the crate import survives.
    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].module, "shared.player");
}

#[test]
fn crate_glob_generates_module_require() {
    let source = r"
use crate::shared::utils::*;

pub fn on_init() {}
";

    let module = must_ok_parse(parse_module(source, "control.on_init"));
    let lua = must_ok(LuaGenerator::new().generate_module(&module));

    // The module is required even though no specific items were named.
    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].module, "shared.utils");
    assert!(lua.contains("require(\"__mod__/lua/shared/utils\")"));
}

#[test]
fn generates_require_for_imports() {
    let source = r"
use crate::shared::player::MyPlayer;

pub fn on_init() {
    let player = MyPlayer::new();
}
";

    let module = must_ok_parse(parse_module(source, "control.on_init"));
    let lua = must_ok(LuaGenerator::new().generate_module(&module));

    assert!(lua.contains("local shared_player = require(\"__mod__/lua/shared/player\")"));
    assert!(lua.contains("local MyPlayer = shared_player.MyPlayer"));
    assert!(lua.contains("local player = MyPlayer.new()"));
}

#[test]
fn inline_crate_path_generates_require() {
    let source = r"
pub fn on_init() {
    let player = crate::shared::player::MyPlayer::new();
}
";

    let module = must_ok_parse(parse_module(source, "control.on_init"));
    let lua = must_ok(LuaGenerator::new().generate_module(&module));

    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].module, "shared.player");
    assert_eq!(module.imports[0].local, "shared_player");
    assert!(lua.contains("local shared_player = require(\"__mod__/lua/shared/player\")"));
    assert!(lua.contains("local player = shared_player.MyPlayer.new()"));
}

#[test]
fn lowers_binding_crate_use_to_foreign_require() {
    let source = r"
use provider_api::shared::api;

pub fn on_init() {
    api::greet();
}
";

    let mut bindings = BTreeMap::new();
    bindings.insert(
        "provider_api".to_string(),
        FactorioBinding {
            crate_name: "provider_api".to_string(),
            mod_name: "provider".to_string(),
            dependencies: vec!["provider >= 0.1.0".to_string()],
            module_root: "lua".to_string(),
            interface: None,
            remote_fns: BTreeSet::new(),
        },
    );

    let lints = LintConfig::allow_all();
    let mut diagnostics = Vec::new();
    let module = must_ok_parse(parse_module_with_options(
        source,
        "control.on_init",
        &ParseOptions::new(&lints).with_bindings(&bindings),
        &mut diagnostics,
    ));
    let lua = must_ok(LuaGenerator::with_mod_name("consumer").generate_module(&module));

    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].module, "shared.api");
    assert_eq!(module.imports[0].factorio_mod.as_deref(), Some("provider"));
    assert!(lua.contains("local api = require(\"__provider__/lua/shared/api\")"));
    assert!(lua.contains("api.greet()"));
}

#[test]
fn lowers_inline_binding_path_to_foreign_require() {
    let source = r"
pub fn on_init() {
    let _v = provider_api::shared::api::VERSION;
}
";

    let mut bindings = BTreeMap::new();
    bindings.insert(
        "provider_api".to_string(),
        FactorioBinding {
            crate_name: "provider_api".to_string(),
            mod_name: "provider".to_string(),
            dependencies: vec!["provider >= 0.1.0".to_string()],
            module_root: "lua".to_string(),
            interface: None,
            remote_fns: BTreeSet::new(),
        },
    );

    let lints = LintConfig::allow_all();
    let mut diagnostics = Vec::new();
    let module = must_ok_parse(parse_module_with_options(
        source,
        "control.on_init",
        &ParseOptions::new(&lints).with_bindings(&bindings),
        &mut diagnostics,
    ));
    let lua = must_ok(LuaGenerator::with_mod_name("consumer").generate_module(&module));

    assert!(lua.contains("require(\"__provider__/lua/shared/api\")"));
    assert!(lua.contains("shared_api.VERSION"));
}

#[test]
fn lowers_remote_binding_call() {
    let source = r#"
use provider_api::remote;

pub fn on_init() {
    remote::greet("hi");
}
"#;

    let mut bindings = BTreeMap::new();
    bindings.insert(
        "provider_api".to_string(),
        FactorioBinding {
            crate_name: "provider_api".to_string(),
            mod_name: "provider".to_string(),
            dependencies: vec!["provider >= 0.1.0".to_string()],
            module_root: "lua".to_string(),
            interface: Some("provider".to_string()),
            remote_fns: BTreeSet::from(["greet".to_string()]),
        },
    );

    let lints = LintConfig::allow_all();
    let mut diagnostics = Vec::new();
    let module = must_ok_parse(parse_module_with_options(
        source,
        "control.on_init",
        &ParseOptions::new(&lints).with_bindings(&bindings),
        &mut diagnostics,
    ));
    let lua = must_ok(LuaGenerator::with_mod_name("consumer").generate_module(&module));

    assert!(
        lua.contains("remote.call(\"provider\", \"greet\", \"hi\")"),
        "generated lua:\n{lua}"
    );
    assert!(!lua.contains("require(\"__provider__/lua/remote\")"));
}

#[test]
fn lowers_flat_root_remote_call() {
    let source = r#"
pub fn on_init() {
    provider_api::greet("hi");
}
"#;

    let mut bindings = BTreeMap::new();
    bindings.insert(
        "provider_api".to_string(),
        FactorioBinding {
            crate_name: "provider_api".to_string(),
            mod_name: "provider".to_string(),
            dependencies: vec!["provider >= 0.1.0".to_string()],
            module_root: "lua".to_string(),
            interface: Some("provider".to_string()),
            remote_fns: BTreeSet::from(["greet".to_string()]),
        },
    );

    let lints = LintConfig::allow_all();
    let mut diagnostics = Vec::new();
    let module = must_ok_parse(parse_module_with_options(
        source,
        "control.on_init",
        &ParseOptions::new(&lints).with_bindings(&bindings),
        &mut diagnostics,
    ));
    let lua = must_ok(LuaGenerator::with_mod_name("consumer").generate_module(&module));

    assert!(
        lua.contains("remote.call(\"provider\", \"greet\", \"hi\")"),
        "generated lua:\n{lua}"
    );
    assert!(module.imports.is_empty(), "flat remotes must not require");
}

#[test]
fn lowers_imported_root_remote_fn() {
    let source = r#"
use provider_api::greet;

pub fn on_init() {
    greet("hi");
}
"#;

    let mut bindings = BTreeMap::new();
    bindings.insert(
        "provider_api".to_string(),
        FactorioBinding {
            crate_name: "provider_api".to_string(),
            mod_name: "provider".to_string(),
            dependencies: vec!["provider >= 0.1.0".to_string()],
            module_root: "lua".to_string(),
            interface: Some("provider".to_string()),
            remote_fns: BTreeSet::from(["greet".to_string()]),
        },
    );

    let lints = LintConfig::allow_all();
    let mut diagnostics = Vec::new();
    let module = must_ok_parse(parse_module_with_options(
        source,
        "control.on_init",
        &ParseOptions::new(&lints).with_bindings(&bindings),
        &mut diagnostics,
    ));
    let lua = must_ok(LuaGenerator::with_mod_name("consumer").generate_module(&module));

    assert!(
        lua.contains("remote.call(\"provider\", \"greet\", \"hi\")"),
        "generated lua:\n{lua}"
    );
    assert!(module.imports.is_empty());
}
