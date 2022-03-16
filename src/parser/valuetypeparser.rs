use super::itemposition::ItemPosition;
use super::parsetree::ParseTree;
use super::proptypesimple::PropTypeSimple;
use super::token::Token;
use super::tokenstream::TokenStream;
use super::treevariant::TreeVariant;
use crate::leaptypes::{Name, SimpleType};

/*

    Full BNF


    PTYPE               := NAME | NAME PT_ARGS_BLOCK
    PT_ARGS_BLOCK       := [ PT_ARGS ]
    PT_ARGS             := PTYPE | PTYPE PT_ARGS

    NAME                := word

*/

#[derive(Debug, Clone)]
pub enum ValueType {
    Simple(SimpleType),
    List(Box<ValueType>),
    LeapType { name: Name, args: Vec<ValueType> },
}

impl From<PropTypeSimple> for ValueType {
    fn from(item: PropTypeSimple) -> Self {
        let mut item = item;
        match item.name.as_str() {
            "str" => Self::Simple(SimpleType::String),
            "int" => Self::Simple(SimpleType::Integer),
            "float" => Self::Simple(SimpleType::Float),
            "bool" => Self::Simple(SimpleType::Boolean),
            "list" => Self::List(Box::new(Self::from(item.args.remove(0)))),
            name => {
                let name = Name::new(name.to_owned(), item.position).unwrap();
                let args = item.args.into_iter().map(Self::from).collect();
                Self::LeapType { name, args }
            }
        }
    }
}

impl ValueType {
    pub fn args(&self) -> Vec<&ValueType> {
        match self {
            ValueType::Simple(_) => vec![],
            ValueType::List(t) => vec![&t],
            ValueType::LeapType { args, .. } => args.iter().collect(),
        }
    }
}

pub struct ValueTypeParser {
    stream: TokenStream,
}

impl ValueTypeParser {
    pub fn parse(data: &str) -> Result<ValueType, ItemPosition<String>> {
        let stream = TokenStream::new(&data);
        let mut parser = ValueTypeParser { stream };
        let tree = parser.parse_ptype()?;
        let value_type = Self::tree_to_prop_type_simple(&tree);
        Ok(ValueType::from(value_type))
    }

    fn parse_ptype(&mut self) -> Result<ParseTree, ItemPosition<String>> {
        let mut tree = ParseTree::new(TreeVariant::PType, self.stream.get().0);
        tree.nodes.push(self.parse_name()?);
        if self.stream.get().1 == Token::BracketLeft {
            tree.nodes.push(self.parse_pt_args_block()?);
        }
        Ok(tree)
    }

    fn parse_pt_args_block(&mut self) -> Result<ParseTree, ItemPosition<String>> {
        let mut tree = ParseTree::new(TreeVariant::PTArgsBlock, self.stream.get().0);
        if self.stream.get().1 == Token::BracketLeft {
            self.stream.next();
            tree.nodes.push(self.parse_pt_args()?);
            if self.stream.get().1 == Token::BracketRight {
                self.stream.next();
            } else {
                return Err(self.stream.get().replace("Expecting `]`".to_owned()));
            }
        } else {
            return Err(self.stream.get().replace("Expecting `[`".to_owned()));
        }
        Ok(tree)
    }

    fn parse_pt_args(&mut self) -> Result<ParseTree, ItemPosition<String>> {
        let mut tree = ParseTree::new(TreeVariant::PTArgs, self.stream.get().0);
        tree.nodes.push(self.parse_ptype()?);
        if let Token::Word(_) = self.stream.get().1 {
            tree.nodes.push(self.parse_pt_args()?);
        }
        Ok(tree)
    }

    fn parse_name(&mut self) -> Result<ParseTree, ItemPosition<String>> {
        match self.stream.consume() {
            ItemPosition(p, Token::Word(w)) => Ok(ParseTree {
                variant: TreeVariant::Name(w.clone()),
                position: *p,
                nodes: vec![],
            }),
            p => Err(p.replace("Expecting name".to_owned())),
        }
    }

    fn tree_to_prop_type_simple(tree: &ParseTree) -> PropTypeSimple {
        // tree -> PType
        let name_node = &tree.nodes[0];
        let name = if let TreeVariant::Name(w) = &name_node.variant {
            w.clone()
        } else {
            panic!("Incorrect parse tree");
        };
        let args = if let Some(t) = tree.nodes.get(1) {
            // t -> PTArgsBlock
            let t = &t.nodes[0];
            // t -> PTArgs
            Self::tree_to_ptargs(t)
        } else {
            vec![]
        };
        PropTypeSimple {
            name,
            args,
            position: tree.position,
        }
    }

    fn tree_to_ptargs(tree: &ParseTree) -> Vec<PropTypeSimple> {
        // tree -> PTArgs
        let mut args = vec![Self::tree_to_prop_type_simple(&tree.nodes[0])];
        if let Some(t) = tree.nodes.get(1) {
            // t -> PTArgs
            args.append(&mut Self::tree_to_ptargs(t));
        }
        args
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_simple_type() {
        let vt = ValueTypeParser::parse("int").unwrap();
        assert!(matches!(vt, ValueType::Simple(SimpleType::Integer)));
    }

    #[test]
    fn test_complex_type() {
        let vt = ValueTypeParser::parse("some-class[int aaa[str]]").unwrap();
        assert!(matches!(vt, ValueType::LeapType { .. }));
        if let ValueType::LeapType { name, args } = vt {
            assert_eq!(name.get(), "some-class");
            assert_eq!(args.len(), 2);
        }
    }
}
