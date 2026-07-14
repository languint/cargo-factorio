use crate::{block::Block, debug::FunctionDebug, r#type::Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub r#type: Type,
    pub source_type: Option<String>,
}

/// Metadata from `#[factorio_rs::export]` / `#[factorio_rs::export(interface = "...")]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportMeta {
    /// Override for the Factorio remote interface name (control-stage exports).
    /// When `None`, the consuming mod's package name is used at emit time.
    pub interface: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub body: Block,
    pub doc: Option<String>,
    pub debug: Option<FunctionDebug>,
    /// Factorio event name when this function is registered with `#[factorio_rs::event(...)]`.
    pub event: Option<String>,
    /// Optional event filter table passed to `script.on_event`.
    pub event_filter: Option<crate::expression::Expression>,
    /// Present when marked with `#[factorio_rs::export]`.
    pub export: Option<ExportMeta>,
}
