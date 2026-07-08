use crate::{expression::Expression, function::Function, structure::Struct, r#type::Type};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    FunctionDecl(Function),
    StructDecl(Struct),
    VariableDecl {
        name: String,
        ty: Type,
        source_type: Option<String>,
        value: Expression,
    },
    Assignment {
        target: Expression,
        value: Expression,
    },
    Conditional {
        condition: Expression,
        then_block: Vec<Self>,
        else_block: Vec<Self>,
    },
    Return(Option<Expression>),
    Expr(Expression),
}
