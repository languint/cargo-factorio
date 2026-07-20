use factorio_codegen::LuaGenerator;
use factorio_ir::{expression::Expression, literal::Literal};

#[test]
fn injects_technology_unlock_recipe_and_unit_ingredient_tuple() {
    let technology = Expression::StructLiteral {
        struct_name: Some("Technology".to_string()),
        fields: vec![
            (
                "name".to_string(),
                Expression::Literal(Literal::String("my-mod-widget".to_string())),
            ),
            (
                "effects".to_string(),
                Expression::Array {
                    elements: vec![Expression::StructLiteral {
                        struct_name: Some("UnlockRecipeEffect".to_string()),
                        fields: vec![(
                            "recipe".to_string(),
                            Expression::Literal(Literal::String("my-mod-widget".to_string())),
                        )],
                    }],
                },
            ),
            (
                "unit".to_string(),
                Expression::StructLiteral {
                    struct_name: Some("TechnologyUnit".to_string()),
                    fields: vec![
                        ("count".to_string(), Expression::Literal(Literal::Int(50))),
                        ("time".to_string(), Expression::Literal(Literal::Float(30.0))),
                        (
                            "ingredients".to_string(),
                            Expression::Array {
                                elements: vec![Expression::StructLiteral {
                                    struct_name: Some("TechnologyUnitIngredient".to_string()),
                                    fields: vec![
                                        (
                                            "name".to_string(),
                                            Expression::Literal(Literal::String(
                                                "automation-science-pack".to_string(),
                                            )),
                                        ),
                                        (
                                            "amount".to_string(),
                                            Expression::Literal(Literal::Int(1)),
                                        ),
                                    ],
                                }],
                            },
                        ),
                    ],
                },
            ),
        ],
    };

    let lua = LuaGenerator::new().generate_expression(&technology);
    assert!(
        lua.contains("type = \"technology\""),
        "expected technology type injection: {lua}"
    );
    assert!(
        lua.contains("type = \"unlock-recipe\""),
        "expected unlock-recipe type injection: {lua}"
    );
    assert!(
        lua.contains("{ \"automation-science-pack\", 1 }"),
        "expected unit ingredient tuple: {lua}"
    );
    assert!(
        lua.contains("recipe = \"my-mod-widget\""),
        "expected unlock recipe field: {lua}"
    );
}
