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
    /// `for _, VAR in pairs(ITER) do BODY end` in Lua.
    ForIn {
        var: String,
        iter: Expression,
        body: Vec<Self>,
    },
    /// `while CONDITION do BODY end` in Lua. Rust `loop { }` lowers with
    /// `condition = true`.
    While {
        condition: Expression,
        body: Vec<Self>,
    },
    /// `goto __continue_N` in Lua (the label `::__continue_N::` is emitted by
    /// the enclosing `ForIn` / `While`).
    Continue,
    /// Lua `break` (exits the innermost loop).
    Break,
}
