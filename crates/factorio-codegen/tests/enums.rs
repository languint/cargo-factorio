mod common;

use common::must_ok;
use factorio_codegen::LuaGenerator;
use factorio_ir::{
    block::Block,
    enumeration::{Enum, EnumVariant, EnumVariantFields},
    expression::Expression,
    function::Function,
    module::Module,
    stage::Stage,
    statement::Statement,
};

#[test]
fn generates_tagged_enum_tables_and_methods() {
    let module = Module {
        name: "messages".to_string(),
        stage: Stage::Control,
        body: Block {
            statements: vec![Statement::EnumDecl(Enum {
                name: "Msg".to_string(),
                variants: vec![
                    EnumVariant {
                        name: "Quit".to_string(),
                        fields: EnumVariantFields::Unit,
                    },
                    EnumVariant {
                        name: "Move".to_string(),
                        fields: EnumVariantFields::Tuple { types: vec![] },
                    },
                ],
                constants: vec![],
                methods: vec![Function {
                    name: "quit".to_string(),
                    params: vec![],
                    body: Block {
                        statements: vec![Statement::Return(Some(Expression::EnumLiteral {
                            enum_name: "Msg".to_string(),
                            variant: "Quit".to_string(),
                            fields: vec![],
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
            })],
        },
        symbols: vec![],
        imports: vec![],
        submodules: vec![],
        locales: vec![],
        pending_locales: vec![],
        vtables: vec![],
    };

    let output = must_ok(LuaGenerator::new().generate_module(&module));
    assert!(output.contains("local Msg = {}"));
    assert!(output.contains("Msg.Quit = { tag = \"Quit\" }"));
    assert!(output.contains("function Msg.quit()"));
    assert!(output.contains("return setmetatable({ tag = \"Quit\" }, { __index = Msg })"));
}

#[test]
fn from_into_attaches_target_enum_metatable() {
    use factorio_ir::{
        enumeration::{Enum, EnumVariant, EnumVariantFields},
        function::{Function, Parameter},
        module::{ImportedItem, ModuleImport, Symbol},
        scope::Scope,
        structure::Struct,
        r#type::Type,
    };

    let module = Module {
        name: "shared.widget".to_string(),
        stage: Stage::Shared,
        body: Block { statements: vec![] },
        imports: vec![ModuleImport {
            module: "shared.frame".to_string(),
            local: "shared_frame".to_string(),
            items: vec![ImportedItem {
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
                        params: vec![Parameter {
                            name: "self".to_string(),
                            r#type: Type::Void,
                            source_type: Some("self".to_string()),
                        }],
                        body: Block {
                            statements: vec![Statement::Return(Some(Expression::EnumLiteral {
                                enum_name: "Widget".to_string(),
                                variant: "Frame".to_string(),
                                fields: vec![(
                                    "_1".to_string(),
                                    Expression::Identifier("self".to_string()),
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
            },
            Symbol {
                scope: Scope::Public,
                statement: Statement::EnumDecl(Enum {
                    name: "Widget".to_string(),
                    variants: vec![EnumVariant {
                        name: "Frame".to_string(),
                        fields: EnumVariantFields::Tuple { types: vec![] },
                    }],
                    constants: vec![],
                    methods: vec![],
                    doc: None,
                    debug: None,
                }),
            },
        ],
    };

    let output = must_ok(LuaGenerator::new().generate_module(&module));
    assert!(
        output.contains(
            "return setmetatable({ tag = \"Frame\", _1 = self }, { __index = sharedWidget.Widget })"
        ),
        "Frame:into must attach Widget metatable; got:\n{output}"
    );
}
