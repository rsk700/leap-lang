use super::position::Position;
use super::treevariant::TreeVariant;

#[derive(Debug)]
pub struct ParseTree {
    pub variant: Position<TreeVariant>,
    pub nodes: Vec<ParseTree>,
}

impl ParseTree {
    pub fn new(variant: Position<TreeVariant>) -> Self {
        Self {
            variant,
            nodes: vec![],
        }
    }
}