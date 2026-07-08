#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDebug {
    pub header_comment: String,
    pub return_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDebug {
    pub header_comment: String,
}
