#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum LuaGeneratorError {
    // It could *technically*, but it's likely bad practice
    #[error("function {0} cannot be `local` and exported!")]
    FunctionLocalAndExported(String),

    #[error("struct {0} cannot be `local` and exported!")]
    StructLocalAndExported(String),

    #[error("failed to get table path for struct: {0}")]
    FailedToGetTablePathForStruct(String),
}

pub type LuaGeneratorResult<T> = Result<T, LuaGeneratorError>;
