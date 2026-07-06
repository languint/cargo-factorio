use crate::{literal::Literal, operator::Operator};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        lhs: Box<Expression>,
        op: Operator,
        rhs: Box<Expression>,
    },
}
