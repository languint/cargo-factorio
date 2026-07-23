use crate::ast::{
    expression::Expression, function::Function, structure::StructField, r#type::Type,
};
use crate::meta::debug::StructDebug;

/// How an enum variant carries its payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumVariantFields {
    /// `Color::Red`
    Unit,
    /// `Msg::Move(x, y)` - positional fields lowered as `_1`, `_2`, ...
    Tuple { types: Vec<Type> },
    /// `Msg::Move { x, y }`
    Named(Vec<StructField>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: EnumVariantFields,
}

/// A user-defined enum: method table plus tagged-table values at runtime.
#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub constants: Vec<(String, Expression)>,
    pub methods: Vec<Function>,
    pub doc: Option<String>,
    pub debug: Option<StructDebug>,
}
