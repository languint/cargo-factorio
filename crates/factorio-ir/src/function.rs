use crate::{block::Block, r#type::Type};

pub struct Parameter {
    pub name: String,
    pub r#type: Type,
}

pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub body: Block,
    pub output_type: Type,
}
