use crate::statement::Statement;

/// A [`Block`] holds a list of [`Statements`][`Statement`]
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}
