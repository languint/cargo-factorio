use crate::{literal::Literal, operator::Operator};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    QualifiedPath {
        segments: Vec<String>,
    },
    FieldAccess {
        base: Box<Self>,
        field: String,
    },
    Call {
        func: Box<Self>,
        args: Vec<Self>,
    },
    MethodCall {
        receiver: Box<Self>,
        method: String,
        args: Vec<Self>,
    },
    StructLiteral {
        fields: Vec<(String, Self)>,
    },
    /// An operation between a `lhs` and a `rhs` with an [`Operator`]
    BinaryOp {
        lhs: Box<Self>,
        op: Operator,
        rhs: Box<Self>,
    },
    /// String interpolation parts joined with `..` in Lua.
    FormatConcat {
        parts: Vec<Self>,
    },
    /// Lua array literal `{ a, b, c }`.
    Array {
        elements: Vec<Self>,
    },
}
