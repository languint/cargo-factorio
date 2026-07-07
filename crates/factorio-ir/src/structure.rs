use crate::{expression::Expression, function::Function, r#type::Type};

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<StructField>,
    pub constants: Vec<(String, Expression)>,
    pub methods: Vec<Function>,
}
