use factorio_ir::operator::Operator;

use crate::LuaGenerator;

impl LuaGenerator {
    /// Generate the lua equivalent for an [`Operator`].
    pub(crate) const fn generate_operator(operator: Operator) -> &'static str {
        match operator {
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "*",
            Operator::Div => "/",
            Operator::Eq => "==",
            Operator::Ne => "~=",
            Operator::Lt => "<",
            Operator::Le => "<=",
            Operator::Gt => ">",
            Operator::Ge => ">=",
        }
    }

    /// Get the precedence for an [`Operator`].
    pub(crate) const fn operator_precedence(operator: Operator) -> u8 {
        match operator {
            Operator::Mul | Operator::Div => 20,
            Operator::Add | Operator::Sub => 19,
            Operator::Eq
            | Operator::Ne
            | Operator::Lt
            | Operator::Le
            | Operator::Gt
            | Operator::Ge => 10,
        }
    }
}
