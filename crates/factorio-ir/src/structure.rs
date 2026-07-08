use crate::{debug::StructDebug, expression::Expression, function::Function, r#type::Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub source_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<StructField>,
    pub constants: Vec<(String, Expression)>,
    pub methods: Vec<Function>,
    pub doc: Option<String>,
    pub debug: Option<StructDebug>,
}
