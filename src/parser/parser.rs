use super::propsimple::PropSimple;
use super::proptypesimple::PropTypeSimple;
use super::token::Token;
use super::tokenstream::TokenStream;
use super::{
    itemposition::ItemPosition, parsetree::ParseTree, patherror::PathError,
    treevariant::TreeVariant,
};
use crate::leaptypes::{LeapEnum, LeapSpec, LeapStruct, LeapType, Name, Prop};
use std::fs;

/*

    Full BNF

    START               := STRUCT_DEF | ENUM_DEF

    STRUCT_DEF          := STRUCT NAME T_ARGS_DEF PROPS_DEF
    T_ARGS_DEF          := [ T_ARGS ] | e
    T_ARGS              := NAME | NAME T_ARGS
    PROPS_DEF           := PROP PROPS_DEF | e
    PROP                := NAME COLON PTYPE

    ENUM_DEF            := ENUM NAME T_ARGS_DEF VARIANTS_DEF
    VARIANTS_DEF        := PTYPE VARIANTS_DEF | e

    PTYPE               := NAME | NAME PT_ARGS_BLOCK
    PT_ARGS_BLOCK       := [ PT_ARGS ]
    PT_ARGS             := PTYPE | PTYPE PT_ARGS

    NAME                := word
    STRUCT              := ".struct"
    ENUM                := ".enum"

*/

// tokenizes and parses Leap files
pub struct Parser {
    stream: TokenStream,
}

// todo: verify error positions
impl Parser {
    pub fn parse_paths_iter<'a, T>(paths: T) -> Result<LeapSpec, PathError>
    where
        T: Iterator<Item = &'a str>,
    {
        fn read_to_string(path: &str) -> Result<(&str, String), PathError> {
            match fs::read_to_string(path) {
                Ok(s) => Ok((path, s)),
                Err(e) => Err(PathError::new(format!("{}", e), path.to_owned(), 0)),
            }
        }
        fn parse((path, data): (&str, String)) -> Result<Vec<(&str, LeapType)>, PathError> {
            Parser::parse(&data)
                .map_err(|e| PathError::new(e.1, path.to_owned(), e.0.start))
                .map(|t| t.into_iter().map(|t| (path, t)).collect())
        }
        let types = paths
            .map(read_to_string)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(parse)
            .collect::<Result<Vec<Vec<_>>, _>>()?
            .into_iter()
            .flatten()
            .map(|(path, mut leap_type)| {
                leap_type.set_path(path.to_owned());
                leap_type
            })
            .collect();
        Ok(LeapSpec::new(types))
    }

    pub fn parse(data: &str) -> Result<Vec<LeapType>, ItemPosition<String>> {
        let stream = TokenStream::new(&data);
        let mut parser = Parser { stream };
        let mut trees = vec![];
        while parser.stream.get().1 != Token::End {
            trees.push(parser.parse_start()?);
        }
        if trees.is_empty() {
            Ok(vec![])
        } else {
            for t in &mut trees {
                t.calc_length();
            }
            trees
                .into_iter()
                .map(|t| Self::tree_to_leaptype(&t))
                .collect::<Result<Vec<LeapType>, ItemPosition<String>>>()
        }
    }

    fn parse_start(&mut self) -> Result<ParseTree, ItemPosition<String>> {
        let mut tree = ParseTree::new(TreeVariant::Start, self.stream.get().0);
        let node = match self.stream.get() {
            ItemPosition(.., Token::Struct) => self.parse_struct_def()?,
            ItemPosition(.., Token::Enum) => self.parse_enum_def()?,
            p => return Err(p.replace("Expecting `.enum` or `.struct`".to_owned())),
        };
        tree.nodes.push(node);
        Ok(tree)
    }

    fn parse_struct_def(&mut self) -> Result<ParseTree, ItemPosition<String>> {
        let mut tree = ParseTree::new(TreeVariant::StructDef, self.stream.get().0);
        if self.stream.get().1 != Token::Struct {
            return Err(self.stream.get().replace("Expecting `.struct`".to_owned()));
        }
        self.stream.next();
        tree.nodes.push(self.parse_name()?);
        tree.nodes.push(self.parse_t_args_def()?);
        tree.nodes.push(self.parse_props_def()?);
        Ok(tree)
    }

    fn parse_t_args_def(&mut self) -> Result<ParseTree, ItemPosition<String>> {
        let mut tree = ParseTree::new(TreeVariant::TArgsDef, self.stream.get().0);
        if self.stream.get().1 == Token::BracketLeft {
            self.stream.next();
            tree.nodes.push(self.parse_t_args()?);
            if self.stream.get().1 == Token::BracketRight {
                self.stream.next();
            } else {
                return Err(self.stream.get().replace("Expecting `]`".to_owned()));
            }
        }
        Ok(tree)
    }

    fn parse_t_args(&mut self) -> Result<ParseTree, ItemPosition<String>> {
        let mut tree = ParseTree::new(TreeVariant::TArgs, self.stream.get().0);
        tree.nodes.push(self.parse_name()?);
        if let Token::Word(_) = self.stream.get().1 {
            tree.nodes.push(self.parse_t_args()?);
        }
        Ok(tree)
    }

    fn parse_props_def(&mut self) -> Result<ParseTree, ItemPosition<String>> {
        let mut tree = ParseTree::new(TreeVariant::PropsDef, self.stream.get().0);
        if let Token::Word(_) = self.stream.get().1 {
            tree.nodes.push(self.parse_prop()?);
            tree.nodes.push(self.parse_props_def()?);
        }
        Ok(tree)
    }

    fn parse_prop(&mut self) -> Result<ParseTree, ItemPosition<String>> {
        let mut tree = ParseTree::new(TreeVariant::Prop, self.stream.get().0);
        tree.nodes.push(self.parse_name()?);
        if let Token::Colon = self.stream.get().1 {
            self.stream.next();
        } else {
            return Err(self.stream.get().replace("Expecting `:`".to_owned()));
        }
        tree.nodes.push(self.parse_ptype()?);
        Ok(tree)
    }

    fn parse_enum_def(&mut self) -> Result<ParseTree, ItemPosition<String>> {
        let mut tree = ParseTree::new(TreeVariant::EnumDef, self.stream.get().0);
        if self.stream.get().1 != Token::Enum {
            return Err(self.stream.get().replace("Expecting `.enum`".to_owned()));
        }
        self.stream.next();
        tree.nodes.push(self.parse_name()?);
        tree.nodes.push(self.parse_t_args_def()?);
        tree.nodes.push(self.parse_variants_def()?);
        Ok(tree)
    }

    fn parse_variants_def(&mut self) -> Result<ParseTree, ItemPosition<String>> {
        let mut tree = ParseTree::new(TreeVariant::VariantsDef, self.stream.get().0);
        if let Token::Word(_) = self.stream.get().1 {
            tree.nodes.push(self.parse_ptype()?);
            tree.nodes.push(self.parse_variants_def()?);
        }
        Ok(tree)
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
                tree.position = tree.position.extend(&self.stream.get().0);
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

    fn tree_to_leaptype(tree: &ParseTree) -> Result<LeapType, ItemPosition<String>> {
        // tree -> Start
        let tree = &tree.nodes[0];
        match tree.variant {
            TreeVariant::StructDef => Ok(LeapType::Struct(Self::tree_to_struct(&tree)?)),
            TreeVariant::EnumDef => Ok(LeapType::Enum(Self::tree_to_enum(&tree)?)),
            _ => panic!("Incorrect parse tree"),
        }
    }

    fn tree_to_struct(tree: &ParseTree) -> Result<LeapStruct, ItemPosition<String>> {
        // tree -> StructDef
        let args = if tree.nodes[1].nodes.is_empty() {
            vec![]
        } else {
            Self::tree_to_args(&tree.nodes[1].nodes[0])?
        };
        let props_simple = if tree.nodes[2].nodes.is_empty() {
            vec![]
        } else {
            Self::tree_to_simple_props(&tree.nodes[2])?
        };
        let props = props_simple
            .into_iter()
            .map(
                |prop_simple| match prop_simple.prop_type_simple.try_into_prop_type(&args) {
                    Ok(prop_type) => Ok(Prop {
                        name: prop_simple.name,
                        prop_type,
                        position: prop_simple.position,
                    }),
                    Err(e) => Err(ItemPosition(prop_simple.position, e)),
                },
            )
            .collect::<Result<_, _>>()?;
        Ok(LeapStruct {
            name: Self::tree_to_name(&tree.nodes[0])?,
            args,
            props,
            // path is unknown at this point
            path: "".to_owned(),
            position: tree.position,
        })
    }

    fn tree_to_args(tree: &ParseTree) -> Result<Vec<Name>, ItemPosition<String>> {
        // tree -> TArgs
        let mut args = vec![];
        let mut tree = tree;
        loop {
            args.push(Self::tree_to_name(&tree.nodes[0])?);
            if tree.nodes.len() == 2 {
                tree = &tree.nodes[1];
            } else {
                break;
            }
        }
        Ok(args)
    }

    fn tree_to_enum(tree: &ParseTree) -> Result<LeapEnum, ItemPosition<String>> {
        // tree -> EnumDef
        let args = if tree.nodes[1].nodes.is_empty() {
            vec![]
        } else {
            Self::tree_to_args(&tree.nodes[1].nodes[0])?
        };
        let variants = if tree.nodes[2].nodes.is_empty() {
            vec![]
        } else {
            Self::tree_to_simple_variants(&tree.nodes[2])
        };
        let variants = variants
            .into_iter()
            .map(|p| {
                let name = match Name::new(p.name.clone(), p.name_position) {
                    Ok(n) => n,
                    Err(e) => return Err(ItemPosition(tree.position, e)),
                };
                let position = p.position;
                match p.try_into_prop_type(&args) {
                    Ok(prop_type) => Ok(Prop {
                        name,
                        prop_type,
                        position,
                    }),
                    Err(e) => Err(ItemPosition(tree.position, e)),
                }
            })
            .collect::<Result<_, _>>()?;
        Ok(LeapEnum {
            name: Self::tree_to_name(&tree.nodes[0])?,
            args,
            variants,
            // path is unknown at this point
            path: "".to_owned(),
            position: tree.position,
        })
    }

    fn tree_to_simple_props(tree: &ParseTree) -> Result<Vec<PropSimple>, ItemPosition<String>> {
        // tree -> PropsDef
        let mut props = vec![];
        let mut tree = tree;
        loop {
            let prop_tree = &tree.nodes[0];
            props.push(PropSimple {
                name: Self::tree_to_name(&prop_tree.nodes[0])?,
                prop_type_simple: Self::tree_to_prop_type_simple(&prop_tree.nodes[1]),
                position: prop_tree.position,
            });
            tree = &tree.nodes[1];
            if tree.nodes.is_empty() {
                break;
            }
        }
        Ok(props)
    }

    fn tree_to_simple_variants(tree: &ParseTree) -> Vec<PropTypeSimple> {
        // tree -> VariantsDef
        let mut variants = vec![];
        let mut tree = tree;
        loop {
            variants.push(Self::tree_to_prop_type_simple(&tree.nodes[0]));
            tree = &tree.nodes[1];
            if tree.nodes.is_empty() {
                break;
            }
        }
        variants
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
            name_position: name_node.position,
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

    fn tree_to_name(tree: &ParseTree) -> Result<Name, ItemPosition<String>> {
        // tree -> Name
        if let TreeVariant::Name(n) = &tree.variant {
            Name::new(n.clone(), tree.position).map_err(|e| ItemPosition(tree.position, e))
        } else {
            panic!("Incorrect parse tree");
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_empty_spec() {
        let types = &Parser::parse("").unwrap();
        assert!(types.is_empty());
    }

    #[test]
    fn test_simple_error() {
        let r = &Parser::parse("aaa bbb ccc");
        assert!(matches!(r, Err(_)));
    }

    #[test]
    fn test_parse_simple_struct() {
        let simple_struct = &Parser::parse(".struct aaa").unwrap()[0];
        assert!(matches!(simple_struct, LeapType::Struct(_)));
        if let LeapType::Struct(s) = simple_struct {
            assert_eq!(s.name.get(), "aaa");
            assert_eq!(s.position.start, 0);
            assert_eq!(s.position.length, 11);
        }
    }

    #[test]
    fn test_parse_struct_with_args() {
        let s = &Parser::parse(".struct aaa[a]").unwrap()[0];
        assert!(matches!(s, LeapType::Struct(_)));
        if let LeapType::Struct(s) = s {
            assert_eq!(s.args.len(), 1);
            assert_eq!(s.position.start, 0);
            assert_eq!(s.position.length, 14);
        }
        let s = &Parser::parse(".struct aaa[a b c]").unwrap()[0];
        assert!(matches!(s, LeapType::Struct(_)));
        if let LeapType::Struct(s) = s {
            assert_eq!(s.args.len(), 3);
        }
    }

    #[test]
    fn test_parse_struct_with_props() {
        let s = &Parser::parse(".struct aaa\n    p1: int").unwrap()[0];
        assert!(matches!(s, LeapType::Struct(_)));
        if let LeapType::Struct(s) = s {
            assert_eq!(s.props.len(), 1);
            assert_eq!(s.position.start, 0);
            assert_eq!(s.position.length, 23);
            let prop = &s.props[0];
            assert_eq!(prop.position.start, 16);
            assert_eq!(prop.position.length, 7);
        }
        let s = &Parser::parse(".struct aaa[a]\n    p1: int").unwrap()[0];
        assert!(matches!(s, LeapType::Struct(_)));
        if let LeapType::Struct(s) = s {
            assert_eq!(s.props.len(), 1);
        }
        let s = &Parser::parse(".struct aaa[a]\n    p1: int\n    p2: str").unwrap()[0];
        assert!(matches!(s, LeapType::Struct(_)));
        if let LeapType::Struct(s) = s {
            assert_eq!(s.props.len(), 2);
        }
        let s = &Parser::parse(
            ".struct aaa[a]\n    p1: int\n    p2: str\n    p3: ccc[a int mm[kk kkk[a] pp]]",
        )
        .unwrap()[0];
        assert!(matches!(s, LeapType::Struct(_)));
        if let LeapType::Struct(s) = s {
            assert_eq!(s.props.len(), 3);
            let prop = &s.props[0];
            assert_eq!(prop.position.start, 19);
            assert_eq!(prop.position.length, 7);
            let prop = &s.props[2];
            assert_eq!(prop.position.start, 43);
            assert_eq!(prop.position.length, 31);
        }
    }

    #[test]
    fn test_parse_simple_enum() {
        let simple_enum = &Parser::parse(".enum aaa").unwrap()[0];
        assert!(matches!(simple_enum, LeapType::Enum(_)));
        if let LeapType::Enum(e) = simple_enum {
            assert_eq!(e.name.get(), "aaa");
            assert_eq!(e.position.start, 0);
            assert_eq!(e.position.length, 9);
        }

        let simple_enum = &Parser::parse(" .enum aaa ").unwrap()[0];
        assert!(matches!(simple_enum, LeapType::Enum(_)));
        if let LeapType::Enum(e) = simple_enum {
            assert_eq!(e.name.get(), "aaa");
            assert_eq!(e.position.start, 1);
            assert_eq!(e.position.length, 9);
        }
    }

    #[test]
    fn test_parse_enum_with_args() {
        let e = &Parser::parse(".enum kkk[a]").unwrap()[0];
        assert!(matches!(e, LeapType::Enum(_)));
        if let LeapType::Enum(e) = e {
            assert_eq!(e.args.len(), 1);
        }
        let e = &Parser::parse(".enum kkk[a b c]").unwrap()[0];
        assert!(matches!(e, LeapType::Enum(_)));
        if let LeapType::Enum(e) = e {
            assert_eq!(e.args.len(), 3);
        }
    }

    #[test]
    fn test_parse_enum_with_props() {
        let e = &Parser::parse(
            "
            .enum kkk
                aaa[a]
        ",
        )
        .unwrap()[0];
        assert!(matches!(e, LeapType::Enum(_)));
        if let LeapType::Enum(e) = e {
            assert_eq!(e.position.start, 13);
            assert_eq!(e.position.length, 32);
            assert_eq!(e.variants.len(), 1);
            let variant = &e.variants[0];
            assert_eq!(variant.position.start, 39);
            assert_eq!(variant.position.length, 6);
        }
        let e = &Parser::parse(
            "
            .enum kkk
                aaa[a]
                bbb
                mmm[b]
        ",
        )
        .unwrap()[0];
        assert!(matches!(e, LeapType::Enum(_)));
        if let LeapType::Enum(e) = e {
            assert_eq!(e.position.start, 13);
            assert_eq!(e.position.length, 75);
            assert_eq!(e.variants.len(), 3);
            let variant = &e.variants[1];
            assert_eq!(variant.position.start, 62);
            assert_eq!(variant.position.length, 3);
            let variant = &e.variants[2];
            assert_eq!(variant.position.start, 82);
            assert_eq!(variant.position.length, 6);
        }
    }

    #[test]
    fn test_err_position_simple() {
        let e = Parser::parse("aaa");
        if let Err(ItemPosition(p, _)) = e {
            assert_eq!(p.start, 0);
        } else {
            panic!("expecting error");
        }
    }

    #[test]
    fn test_err_position2() {
        let e = Parser::parse(".struct aaa[]");
        if let Err(ItemPosition(p, _)) = e {
            assert_eq!(p.start, 12);
        } else {
            panic!("expecting error");
        }
    }
}
