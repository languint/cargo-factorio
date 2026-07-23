mod common;

use common::must_ok;
use factorio_codegen::LuaGenerator;
use factorio_ir::{
    block::Block,
    expression::Expression,
    function::Function,
    literal::Literal,
    module::{Module, Symbol},
    scope::Scope,
    stage::Stage,
    statement::Statement,
    structure::{Struct, StructField},
    r#type::Type,
};

#[test]
fn rewrites_associated_paths_inside_struct_methods() {
    let module = Module {
        name: "player".to_string(),
        stage: Stage::Control,
        body: Block { statements: vec![] },
        imports: vec![],
        submodules: vec![],
        locales: vec![],
        pending_locales: vec![],
        vtables: vec![],
        symbols: vec![Symbol {
            scope: Scope::Public,
            statement: Statement::StructDecl(Struct {
                name: "MyPlayer".to_string(),
                fields: vec![StructField {
                    name: "health".to_string(),
                    ty: Type::Int,
                    source_type: None,
                }],
                constants: vec![(
                    "DEFAULT_HEALTH".to_string(),
                    Expression::Literal(Literal::Int(100)),
                )],
                methods: vec![Function {
                    name: "new".to_string(),
                    params: vec![],
                    body: Block {
                        statements: vec![Statement::Return(Some(Expression::StructLiteral {
                            struct_name: None,
                            fields: vec![(
                                "health".to_string(),
                                Expression::QualifiedPath {
                                    segments: vec![
                                        "MyPlayer".to_string(),
                                        "DEFAULT_HEALTH".to_string(),
                                    ],
                                },
                            )],
                        }))],
                    },
                    doc: None,
                    debug: None,
                    event: None,
                    event_filter: None,
                    export: None,
                    inline: false,
                }],
                doc: None,
                debug: None,
            }),
        }],
    };

    let output = must_ok(LuaGenerator::new().generate_module(&module));

    assert!(
        output.contains("local __mt_MyPlayer = { __index = player.MyPlayer }"),
        "{output}"
    );
    assert!(
        output.contains(
            "return setmetatable({ health = player.MyPlayer.DEFAULT_HEALTH }, __mt_MyPlayer)"
        ),
        "{output}"
    );
}

#[test]
fn self_field_reads_use_rawget_inside_struct_methods() {
    let module = Module {
        name: "shared.frame".to_string(),
        stage: Stage::Control,
        body: Block { statements: vec![] },
        imports: vec![],
        submodules: vec![],
        locales: vec![],
        pending_locales: vec![],
        vtables: vec![],
        symbols: vec![Symbol {
            scope: Scope::Public,
            statement: Statement::StructDecl(Struct {
                name: "Frame".to_string(),
                fields: vec![StructField {
                    name: "name".to_string(),
                    ty: Type::Str,
                    source_type: None,
                }],
                constants: vec![],
                methods: vec![Function {
                    name: "name".to_string(),
                    params: vec![
                        factorio_ir::function::Parameter {
                            name: "self".to_string(),
                            r#type: Type::Void,
                            source_type: None,
                        },
                        factorio_ir::function::Parameter {
                            name: "name".to_string(),
                            r#type: Type::Str,
                            source_type: None,
                        },
                    ],
                    body: Block {
                        statements: vec![Statement::Return(Some(Expression::FieldAccess {
                            base: Box::new(Expression::Identifier("self".to_string())),
                            field: "name".to_string(),
                        }))],
                    },
                    doc: None,
                    debug: None,
                    event: None,
                    event_filter: None,
                    export: None,
                    inline: false,
                }],
                doc: None,
                debug: None,
            }),
        }],
    };

    let output = must_ok(LuaGenerator::new().generate_module(&module));
    assert!(
        output.contains("return rawget(self, \"name\")"),
        "expected rawget for self.field inside methods, got:\n{output}"
    );
    assert!(
        !output.contains("return self.name"),
        "self.name must not be used (collides with :name method):\n{output}"
    );
}

#[test]
fn self_field_writes_use_rawset_inside_struct_methods() {
    let module = Module {
        name: "shared.frame".to_string(),
        stage: Stage::Control,
        body: Block { statements: vec![] },
        imports: vec![],
        submodules: vec![],
        locales: vec![],
        pending_locales: vec![],
        vtables: vec![],
        symbols: vec![Symbol {
            scope: Scope::Public,
            statement: Statement::StructDecl(Struct {
                name: "Frame".to_string(),
                fields: vec![StructField {
                    name: "name".to_string(),
                    ty: Type::Str,
                    source_type: None,
                }],
                constants: vec![],
                methods: vec![Function {
                    name: "with_name".to_string(),
                    params: vec![
                        factorio_ir::function::Parameter {
                            name: "self".to_string(),
                            r#type: Type::Void,
                            source_type: None,
                        },
                        factorio_ir::function::Parameter {
                            name: "name".to_string(),
                            r#type: Type::Str,
                            source_type: None,
                        },
                    ],
                    body: Block {
                        statements: vec![Statement::Assignment {
                            target: Expression::FieldAccess {
                                base: Box::new(Expression::Identifier("self".to_string())),
                                field: "name".to_string(),
                            },
                            value: Expression::Identifier("name".to_string()),
                        }],
                    },
                    doc: None,
                    debug: None,
                    event: None,
                    event_filter: None,
                    export: None,
                    inline: false,
                }],
                doc: None,
                debug: None,
            }),
        }],
    };

    let output = must_ok(LuaGenerator::new().generate_module(&module));
    assert!(
        output.contains("rawset(self, \"name\", name)"),
        "expected rawset for self.field writes, got:\n{output}"
    );
    assert!(
        !output.contains("rawget(self, \"name\") ="),
        "must not assign to rawget(...):\n{output}"
    );
}

#[test]
fn qualifies_same_module_type_paths_outside_own_method() {
    let module = Module {
        name: "shared.widget".to_string(),
        stage: Stage::Control,
        body: Block { statements: vec![] },
        imports: vec![factorio_ir::module::ModuleImport {
            module: "shared.frame".to_string(),
            local: "shared_frame".to_string(),
            items: vec![factorio_ir::module::ImportedItem {
                name: "Frame".to_string(),
                local: "Frame".to_string(),
            }],
            factorio_mod: None,
            module_root: None,
        }],
        submodules: vec![],
        locales: vec![],
        pending_locales: vec![],
        vtables: vec![],
        symbols: vec![
            Symbol {
                scope: Scope::Public,
                statement: Statement::StructDecl(Struct {
                    name: "Frame".to_string(),
                    fields: vec![],
                    constants: vec![],
                    methods: vec![Function {
                        name: "into".to_string(),
                        params: vec![factorio_ir::function::Parameter {
                            name: "self".to_string(),
                            r#type: Type::Int,
                            source_type: None,
                        }],
                        body: Block {
                            statements: vec![Statement::Return(Some(Expression::Call {
                                func: Box::new(Expression::QualifiedPath {
                                    segments: vec!["Widget".to_string(), "from_frame".to_string()],
                                }),
                                args: vec![Expression::Identifier("self".to_string())],
                            }))],
                        },
                        doc: None,
                        debug: None,
                        event: None,
                        event_filter: None,
                        export: None,
                        inline: false,
                    }],
                    doc: None,
                    debug: None,
                }),
            },
            Symbol {
                scope: Scope::Public,
                statement: Statement::StructDecl(Struct {
                    name: "Widget".to_string(),
                    fields: vec![],
                    constants: vec![],
                    methods: vec![Function {
                        name: "from_frame".to_string(),
                        params: vec![factorio_ir::function::Parameter {
                            name: "frame".to_string(),
                            r#type: Type::Int,
                            source_type: None,
                        }],
                        body: Block {
                            statements: vec![Statement::Return(Some(Expression::Identifier(
                                "frame".to_string(),
                            )))],
                        },
                        doc: None,
                        debug: None,
                        event: None,
                        event_filter: None,
                        export: None,
                        inline: false,
                    }],
                    doc: None,
                    debug: None,
                }),
            },
        ],
    };

    let output = must_ok(LuaGenerator::new().generate_module(&module));
    assert!(
        output.contains("return sharedWidget.Widget.from_frame(self)"),
        "expected same-module Widget path to be qualified, got:\n{output}"
    );
}
