mod common;

use common::must_ok_parse;
use factorio_frontend::{discover_modules, parse_discovered_module};
use factorio_ir::statement::Statement;
use std::path::Path;

#[test]
fn module_level_export_applies_to_pub_fns() {
    let source = r#"
#[factorio_rs::export]
#[factorio_rs::control]
mod control {
    pub fn greet(name: &str) {
        let _ = name;
    }

    fn private_helper() {}

    #[factorio_rs::export(interface = "custom")]
    pub fn special() {}
}
"#;

    let modules = discover_modules(
        Path::new("/project/src"),
        Path::new("/project/src/lib.rs"),
        source,
    )
    .expect("discover");
    assert_eq!(modules.len(), 1);
    assert!(modules[0].default_export.is_some());

    let module = must_ok_parse(parse_discovered_module(&modules[0]));
    let exports: Vec<_> = module
        .symbols
        .iter()
        .filter_map(|symbol| match &symbol.statement {
            Statement::FunctionDecl(func) => Some((func.name.as_str(), func.export.as_ref())),
            _ => None,
        })
        .collect();

    assert_eq!(exports.len(), 2);
    let greet = exports.iter().find(|(name, _)| *name == "greet").unwrap();
    assert!(greet.1.is_some());
    assert_eq!(greet.1.unwrap().interface, None);

    let special = exports.iter().find(|(name, _)| *name == "special").unwrap();
    assert_eq!(special.1.unwrap().interface.as_deref(), Some("custom"));
}

#[test]
fn bare_interface_flag_parses_as_default_export() {
    let source = r#"
#[factorio_rs::control]
mod control {
    #[factorio_rs::export(interface)]
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }
}
"#;

    let modules = discover_modules(
        Path::new("/project/src"),
        Path::new("/project/src/lib.rs"),
        source,
    )
    .expect("discover");
    let module = must_ok_parse(parse_discovered_module(&modules[0]));
    let Statement::FunctionDecl(func) = &module.symbols[0].statement else {
        panic!("expected function");
    };
    assert!(func.export.is_some());
    assert_eq!(func.export.as_ref().unwrap().interface, None);
}

#[test]
fn nested_export_mod_exports_pub_fns() {
    let source = r#"
#[factorio_rs::control]
mod control {
    #[factorio_rs::export]
    mod api {
        pub fn greet(name: &str) {
            let _ = name;
        }

        pub fn farewell() {}
    }
}
"#;

    let modules = discover_modules(
        Path::new("/project/src"),
        Path::new("/project/src/lib.rs"),
        source,
    )
    .expect("discover");
    let module = must_ok_parse(parse_discovered_module(&modules[0]));
    let names: Vec<_> = module
        .symbols
        .iter()
        .filter_map(|symbol| match &symbol.statement {
            Statement::FunctionDecl(func) if func.export.is_some() => Some(func.name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(names, vec!["greet", "farewell"]);
}
