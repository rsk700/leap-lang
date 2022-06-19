// LeapType
// LeapEnum
// Name - StrongString - length, allowed symbols
// todo: rename file to leapspec.rs? file per struct/enum?
// todo: checks - enum variants should have unique names (eg. if need multiple variants of same type wrap in struct first)
// todo: only allow structs to be variants of enum
// todo: checks - type args should be unique relative to struct and enum names, same type arg names can be used in different types
use crate::handle::Handle;
use crate::naming;
use crate::parser::position::Position;
use crate::prop_recursion_check::PropRecursionCheck;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

// todo: trait Name to String
#[derive(Debug, Clone)]
pub struct Name {
    name: String,
    alias: Option<String>,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimpleType {
    String,
    Integer,
    Float,
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    Simple(SimpleType),
    List(Box<ValueType>),
    TypeArg(Name),
    LeapType { name: Name, args: Vec<ValueType> },
}

// todo: rename -> Property
#[derive(Debug)]
pub struct Prop {
    pub name: Name,
    pub prop_type: ValueType,
    pub position: Position,
    pub is_recursive: bool,
}

#[derive(Debug)]
pub struct LeapStruct {
    pub name: Name,
    pub args: Vec<Name>,
    pub props: Vec<Prop>,
    pub path: String,
    pub position: Position,
}

#[derive(Debug)]
pub struct LeapEnum {
    pub name: Name,
    pub args: Vec<Name>,
    pub variants: Vec<Prop>,
    pub path: String,
    pub position: Position,
}

#[derive(Debug)]
pub enum LeapType {
    Struct(LeapStruct),
    Enum(LeapEnum),
}

pub type LeapTypeHandle = Handle<LeapType>;

#[derive(Debug)]
pub struct LeapSpec {
    types: Vec<LeapType>,
    name_to_type: HashMap<String, LeapTypeHandle>,
}

#[derive(Debug)]
pub struct Comment {
    pub comment: String,
    pub comment_type: CommentType,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommentType {
    // comment takes full line
    Line,
    // empty line
    Separator,
    // comment follows some code on the same line
    Trail,
}

fn aliased_from_aliases(name: &Name, aliases: &HashMap<String, String>) -> Result<Name, String> {
    name.to_aliased_if_some(aliases.get(name.get()).cloned())
}

impl Name {
    // todo: accept any name here? check names later, to simplify parsing (name can return error in parsing code)
    pub fn new(name: String, position: Position) -> Result<Self, String> {
        // todo: checks
        // - min, max length?
        // - allowed symbols? only control use of delimiter `-` (eg. can be only in the middle, no repeating)?
        // - allowed start symbol?
        // - allowed end symbol?
        Ok(Name {
            name,
            alias: None,
            position,
        })
    }

    pub fn to_aliased(&self, alias: String) -> Result<Self, String> {
        // todo: check alias for same `name` rules
        // todo: when adding alias check there is no same name/alias, global type scoped, property names scoped, separate aliases for Types and for Props?
        Ok(Name {
            alias: Some(alias),
            ..self.clone()
        })
    }

    pub fn to_aliased_if_some(&self, alias: Option<String>) -> Result<Self, String> {
        if let Some(a) = alias {
            self.to_aliased(a)
        } else {
            Ok(Self::new(self.name.clone(), self.position)?)
        }
    }

    pub fn get(&self) -> &str {
        &self.name
    }

    fn get_aliased(&self) -> &str {
        if let Some(alias) = &self.alias {
            alias
        } else {
            &self.name
        }
    }

    pub fn apply_style(&self, style: naming::WritingStyle, separator: &str) -> String {
        naming::apply_style(style, separator, &naming::get_parts(self.get_aliased()))
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Name {}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl SimpleType {
    pub fn name(&self) -> String {
        match self {
            // todo: replace strings with constants
            SimpleType::String => "str".to_owned(),
            SimpleType::Integer => "int".to_owned(),
            SimpleType::Float => "float".to_owned(),
            SimpleType::Boolean => "bool".to_owned(),
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Simple(t) => match t {
                SimpleType::String => write!(f, "str"),
                SimpleType::Integer => write!(f, "int"),
                SimpleType::Float => write!(f, "float"),
                SimpleType::Boolean => write!(f, "bool"),
            },
            Self::List(t) => write!(f, "list[{}]", t),
            Self::TypeArg(n) => write!(f, "{}?", n),
            Self::LeapType { name, args } => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    let args = args
                        .iter()
                        .map(|a| format!("{}", a))
                        .collect::<Vec<_>>()
                        .join(" ");
                    write!(f, "{}[{}]", name, args)
                }
            }
        }
    }
}

impl ValueType {
    pub fn to_aliased(&self, aliases: &HashMap<String, String>) -> Result<Self, String> {
        match self {
            Self::List(t) => Ok(Self::List(Box::new(t.to_aliased(aliases)?))),
            Self::TypeArg(n) => Ok(Self::TypeArg(aliased_from_aliases(n, aliases)?)),
            Self::LeapType { name, args } => Ok(Self::LeapType {
                name: aliased_from_aliases(name, aliases)?,
                args: args
                    .iter()
                    .map(|a| a.to_aliased(aliases))
                    .collect::<Result<_, _>>()?,
            }),
            _ => Ok(self.clone()),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Simple(t) => t.name(),
            Self::List(_) => "list".to_owned(),
            Self::TypeArg(n) => n.get().to_owned(),
            Self::LeapType { name, .. } => name.get().to_owned(),
        }
    }

    pub fn args(&self) -> Vec<ValueType> {
        match self {
            Self::Simple(_) | Self::TypeArg(_) => vec![],
            Self::List(t) => vec![t.as_ref().clone()],
            Self::LeapType { args, .. } => args.clone(),
        }
    }

    pub fn apply_args(&self, applied_args: &HashMap<&Name, &ValueType>) -> Self {
        match self {
            Self::Simple(_) => self.clone(),
            Self::List(t) => Self::List(Box::new(t.apply_args(applied_args))),
            Self::TypeArg(name) => (*applied_args.get(name).unwrap()).clone(),
            Self::LeapType { name, args } => Self::LeapType {
                name: name.clone(),
                args: args.iter().map(|a| a.apply_args(applied_args)).collect(),
            },
        }
    }
}

impl fmt::Display for Prop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.prop_type)
    }
}

impl Prop {
    pub fn to_aliased(&self, aliases: &HashMap<String, String>) -> Result<Self, String> {
        Ok(Self {
            name: aliased_from_aliases(&self.name, aliases)?,
            prop_type: self.prop_type.to_aliased(aliases)?,
            position: self.position,
            is_recursive: self.is_recursive,
        })
    }

    pub fn apply_args(&self, applied_args: &HashMap<&Name, &ValueType>) -> Self {
        Self {
            name: self.name.clone(),
            prop_type: self.prop_type.apply_args(applied_args),
            position: self.position,
            is_recursive: self.is_recursive,
        }
    }
}

impl fmt::Display for LeapStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args = self
            .args
            .iter()
            .map(|a| a.get())
            .collect::<Vec<_>>()
            .join(" ");
        let props = self
            .props
            .iter()
            .map(|p| format!("{}", p))
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            f,
            "Struct({} args: [{}], props: [{}])",
            self.name, args, props
        )
    }
}

impl LeapStruct {
    pub fn to_aliased(&self, aliases: &HashMap<String, String>) -> Result<Self, String> {
        Ok(Self {
            name: aliased_from_aliases(&self.name, aliases)?,
            args: self
                .args
                .iter()
                .map(|a| aliased_from_aliases(a, aliases))
                .collect::<Result<_, _>>()?,
            props: self
                .props
                .iter()
                .map(|p| p.to_aliased(aliases))
                .collect::<Result<_, _>>()?,
            path: self.path.clone(),
            position: self.position,
        })
    }

    pub fn map_args<'a>(&'a self, applied_args: &'a [ValueType]) -> HashMap<&Name, &ValueType> {
        let mut args_map = HashMap::new();
        for (i, name) in self.args.iter().enumerate() {
            // applied_args should have same length as self.args
            args_map.insert(name, &applied_args[i]);
        }
        args_map
    }

    pub fn apply_args(&self, applied_args: &HashMap<&Name, &ValueType>) -> Self {
        Self {
            name: self.name.clone(),
            // as type args was applied there is no type args any more
            args: vec![],
            props: self
                .props
                .iter()
                .map(|p| p.apply_args(applied_args))
                .collect(),
            path: self.path.clone(),
            position: self.position,
        }
    }

    pub fn expand_args(&self, applied_args: &HashMap<&Name, &ValueType>) -> Vec<ValueType> {
        self.args
            .iter()
            .map(|a| (*applied_args.get(a).unwrap()).clone())
            .collect()
    }
}

impl fmt::Display for LeapEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args = self
            .args
            .iter()
            .map(|a| a.get())
            .collect::<Vec<_>>()
            .join(" ");
        let variants = self
            .variants
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            f,
            "Enum({} args: [{}], variants: [{}])",
            self.name, args, variants
        )
    }
}

impl LeapEnum {
    pub fn to_aliased(&self, aliases: &HashMap<String, String>) -> Result<Self, String> {
        Ok(Self {
            name: aliased_from_aliases(&self.name, aliases)?,
            args: self
                .args
                .iter()
                .map(|a| aliased_from_aliases(a, aliases))
                .collect::<Result<_, _>>()?,
            variants: self
                .variants
                .iter()
                .map(|v| v.to_aliased(aliases))
                .collect::<Result<_, _>>()?,
            path: self.path.clone(),
            position: self.position,
        })
    }

    pub fn expand_args(&self, applied_args: &HashMap<&Name, &ValueType>) -> Vec<ValueType> {
        self.args
            .iter()
            .map(|a| (*applied_args.get(a).unwrap()).clone())
            .collect()
    }

    pub fn map_args<'a>(&'a self, applied_args: &'a [ValueType]) -> HashMap<&Name, &ValueType> {
        let mut args_map = HashMap::new();
        for (i, name) in self.args.iter().enumerate() {
            // applied_args should have same length as self.args
            args_map.insert(name, &applied_args[i]);
        }
        args_map
    }

    pub fn apply_args(&self, applied_args: &HashMap<&Name, &ValueType>) -> Self {
        Self {
            name: self.name.clone(),
            // as type args was applied there is no type args any more
            args: vec![],
            variants: self
                .variants
                .iter()
                .map(|v| v.apply_args(applied_args))
                .collect(),
            path: self.path.clone(),
            position: self.position,
        }
    }
}

impl fmt::Display for LeapType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Struct(t) => write!(f, "Type({})", t),
            Self::Enum(e) => write!(f, "Type({})", e),
        }
    }
}

impl LeapType {
    pub fn unwrap_struct_ref(&self) -> &LeapStruct {
        if let LeapType::Struct(s) = self {
            s
        } else {
            panic!("not a struct variant")
        }
    }

    pub fn unwrap_enum_ref(&self) -> &LeapEnum {
        if let LeapType::Enum(e) = self {
            e
        } else {
            panic!("not an enum variant")
        }
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, LeapType::Struct(_))
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, LeapType::Enum(_))
    }

    pub fn to_aliased(&self, aliases: &HashMap<String, String>) -> Result<Self, String> {
        Ok(match self {
            Self::Struct(s) => Self::Struct(s.to_aliased(aliases)?),
            Self::Enum(e) => Self::Enum(e.to_aliased(aliases)?),
        })
    }

    pub fn name(&self) -> &Name {
        match self {
            Self::Enum(e) => &e.name,
            Self::Struct(s) => &s.name,
        }
    }

    pub fn args(&self) -> &[Name] {
        match self {
            Self::Enum(e) => &e.args,
            Self::Struct(s) => &s.args,
        }
    }

    pub fn path(&self) -> &str {
        match self {
            Self::Enum(e) => &e.path,
            Self::Struct(s) => &s.path,
        }
    }

    pub fn set_path(&mut self, path: String) {
        match self {
            Self::Enum(e) => e.path = path,
            Self::Struct(s) => s.path = path,
        }
    }

    pub fn position(&self) -> &Position {
        match self {
            Self::Enum(e) => &e.position,
            Self::Struct(s) => &s.position,
        }
    }

    pub fn expand_args(&self, applied_args: &HashMap<&Name, &ValueType>) -> Vec<ValueType> {
        match self {
            Self::Enum(e) => e.expand_args(applied_args),
            Self::Struct(s) => s.expand_args(applied_args),
        }
    }

    pub fn apply_args(&self, args: &[ValueType]) -> Self {
        match self {
            LeapType::Struct(s) => LeapType::Struct(s.apply_args(&s.map_args(args))),
            LeapType::Enum(e) => LeapType::Enum(e.apply_args(&e.map_args(args))),
        }
    }
}

impl fmt::Display for LeapSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Spec({})",
            self.types
                .iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl IntoIterator for LeapSpec {
    type Item = LeapType;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.types.into_iter()
    }
}

impl LeapSpec {
    pub fn new(types: Vec<LeapType>) -> Self {
        let mut spec = Self {
            types: vec![],
            name_to_type: HashMap::new(),
        };
        for leap_type in types.into_iter() {
            spec.push_type(leap_type);
        }
        spec
    }

    fn push_type(&mut self, leap_type: LeapType) {
        let name = leap_type.name().get().to_owned();
        self.types.push(leap_type);
        self.name_to_type
            .insert(name, LeapTypeHandle::new((self.types.len() - 1) as u32));
    }

    pub fn iter_type_refs(&self) -> impl Iterator<Item = &LeapType> {
        self.types.iter()
    }

    pub fn iter_types(&self) -> impl Iterator<Item = LeapTypeHandle> {
        (0..self.types.len()).map(|i| LeapTypeHandle::new(i as u32))
    }

    pub fn join(&mut self, other: LeapSpec) {
        // todo: consume self, and return new spec? so new spec always created with `new`
        for leap_type in other.into_iter() {
            self.push_type(leap_type);
        }
    }

    pub fn get_type_ref(&self, handle: LeapTypeHandle) -> &LeapType {
        &self.types[handle.as_index()]
    }

    pub fn get_type_mut(&mut self, handle: LeapTypeHandle) -> &mut LeapType {
        &mut self.types[handle.as_index()]
    }

    pub fn get_type_by_name(&self, name: &str) -> Option<LeapTypeHandle> {
        self.name_to_type.get(name).copied()
    }

    pub fn is_struct_name(&self, name: &str) -> bool {
        if let Some(h) = self.get_type_by_name(name) {
            self.get_type_ref(h).is_struct()
        } else {
            false
        }
    }

    pub fn is_enum_name(&self, name: &str) -> bool {
        if let Some(h) = self.get_type_by_name(name) {
            self.get_type_ref(h).is_enum()
        } else {
            false
        }
    }

    pub fn apply_args(&self, handle: LeapTypeHandle, args: &[ValueType]) -> LeapType {
        self.get_type_ref(handle).apply_args(args)
    }

    pub fn to_aliased(&self, aliases: &HashMap<String, String>) -> Result<Self, String> {
        Ok(Self::new(
            self.types
                .iter()
                .map(|t| t.to_aliased(aliases))
                .collect::<Result<_, _>>()?,
        ))
    }

    pub fn mark_recursive_props(&mut self) {
        for h in self.iter_types() {
            let mut recursive_props = vec![];
            let t = self.get_type_ref(h);
            match t {
                LeapType::Struct(s) => {
                    for (i, p) in s.props.iter().enumerate() {
                        if PropRecursionCheck::is_recursive(self, t, p) {
                            recursive_props.push(i);
                        }
                    }
                }
                LeapType::Enum(e) => {
                    for (i, v) in e.variants.iter().enumerate() {
                        if PropRecursionCheck::is_recursive(self, t, v) {
                            recursive_props.push(i);
                        }
                    }
                }
            }
            if !recursive_props.is_empty() {
                let props = match self.get_type_mut(h) {
                    LeapType::Struct(s) => &mut s.props,
                    LeapType::Enum(e) => &mut e.variants,
                };
                for i in recursive_props {
                    props[i].is_recursive = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::parser::Parser;

    use super::*;

    #[test]
    fn test_simple() {
        let spec_text = "
            .struct s1
                a: s2
                b: s3

            .struct s2
                a: s1

            .struct s3
                a: str
        ";
        let mut spec = LeapSpec::new(Parser::parse(spec_text).unwrap());
        spec.mark_recursive_props();
        let h = spec.get_type_by_name("s1").unwrap();
        let s = spec.get_type_ref(h).unwrap_struct_ref();
        assert!(s.props[0].is_recursive);
        assert!(!s.props[1].is_recursive);
    }
}
