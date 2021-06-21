use super::position::Position;
use super::treevariant::TreeVariant;

#[derive(Debug)]
pub struct ParseTree {
    pub variant: TreeVariant,
    pub position: Position,
    pub nodes: Vec<ParseTree>,
}

impl ParseTree {
    pub fn new(variant: TreeVariant, position: Position) -> Self {
        Self {
            variant,
            position,
            nodes: vec![],
        }
    }

    pub fn calc_length(&mut self) {
        // first calculate lenght for nodes, in order to use correc position for extending self.position
        for n in &mut self.nodes {
            n.calc_length();
        }
        if let Some(last) = self.nodes.last() {
            self.position = self.position.extend(&last.position);
        }
    }
}
