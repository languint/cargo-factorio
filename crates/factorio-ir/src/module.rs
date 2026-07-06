use crate::block::Block;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub body: Block,
}
