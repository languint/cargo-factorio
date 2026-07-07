use crate::{literal::Literal, operator::Operator};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        lhs: Box<Self>,
        op: Operator,
        rhs: Box<Self>,
    },
}
