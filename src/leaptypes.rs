// LeapType
// LeapEnum
// Name - StrongString - length, allowed symbols
// todo: rename file to leapspec.rs? file per struct/enum?
// todo: checks - enum variants should have unique names (eg. if need multiple variants of same type wrap in struct first)
// todo: only allow structs to be variants of enum
// todo: checks - type args should be unique relative to struct and enum names, same type arg names can be used in different types
use crate::naming;
use std::collections::HashMap;
use std::fmt;

// todo: trait Name to String
#[derive(Debug, Clone)]
pub struct Name {
    name: String,
    alias: Option<String>,
    // position index of this name in source file
    pub position: usize,
}

#[derive(Debug, Clone)]
pub enum SimpleType {
    String,
    Integer,
    Float,
    Boolean,
}

#[derive(Debug, Clone)]
pub enum PropType {
    Simple(SimpleType),
    List(Box<PropType>),
    TypeArg(Name),
    LeapType { name: Name, args: Vec<PropType> },
}

#[derive(Debug)]
pub struct Prop {
    pub name: Name,
    pub prop_type: PropType,
}

#[derive(Debug)]
pub struct LeapStruct {
    pub name: Name,
    pub args: Vec<Name>,
    pub props: Vec<Prop>,
}

#[derive(Debug)]
pub struct LeapEnum {
    pub name: Name,
    pub args: Vec<Name>,
    pub variants: Vec<Prop>,
}

// todo: info for location of type, names, props for error reporting?
#[derive(Debug)]
pub enum LeapType {
    Struct(LeapStruct),
    Enum(LeapEnum),
}

#[derive(Debug)]
pub struct LeapTypePath {
    pub leap_type: LeapType,
    pub path: String,
}

#[derive(Debug)]
pub struct LeapSpec(Vec<LeapTypePath>);

#[derive(Debug)]
pub struct Comment {
    pub comment: String,
    pub comment_type: CommentType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommentType {
    // comment takes full line
    Line,
    // comment takes full line and have empty line after comment
    LineAndSeparator,
    // comment follows some code on the same line
    Trail,
}

fn get_alias_of_name(name: &Name, aliases: &HashMap<String, String>) -> Option<String> {
    if let Some(a) = aliases.get(name.get()) {
        Some(a.clone())
    } else {
        None
    }
}

fn aliased_from_aliases(name: &Name, aliases: &HashMap<String, String>) -> Result<Name, String> {
    name.to_aliased_if_some(get_alias_of_name(name, aliases))
}

impl Name {
    pub fn new(name: String, position: usize) -> Result<Self, String> {
        // todo: checks
        // - min, max length?
        // - allowed symbols?
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

    pub fn snake_case(&self) -> String {
        naming::snake_case(&naming::get_parts(&self.get_aliased()))
    }

    pub fn upper_camel_case(&self) -> String {
        naming::upper_camel_case(&naming::get_parts(&self.get_aliased()))
    }

    pub fn camel_case(&self) -> String {
        naming::camel_case(&naming::get_parts(&self.get_aliased()))
    }

    pub fn joined(&self) -> String {
        naming::joined(&naming::get_parts(&self.get_aliased()))
    }

    pub fn uppercase_joined(&self) -> String {
        naming::uppercase_joined(&naming::get_parts(&self.get_aliased()))
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

impl fmt::Display for PropType {
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

impl PropType {
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
        })
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
        })
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
        })
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
}

impl fmt::Display for LeapSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Spec({})",
            self.0
                .iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl IntoIterator for LeapSpec {
    type Item = LeapTypePath;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl LeapSpec {
    pub fn new(types: Vec<LeapTypePath>) -> Self {
        Self(types)
    }

    pub fn iter_types(&self) -> impl Iterator<Item = &LeapType> {
        self.0.iter().map(|t| &t.leap_type)
    }

    pub fn to_aliased(&self, aliases: &HashMap<String, String>) -> Result<Self, String> {
        Ok(Self(
            self.0
                .iter()
                .map(|t| t.to_aliased(aliases))
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl fmt::Display for LeapTypePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.leap_type)
    }
}

impl LeapTypePath {
    pub fn new(leap_type: LeapType, path: String) -> Self {
        Self { leap_type, path }
    }

    pub fn to_aliased(&self, aliases: &HashMap<String, String>) -> Result<Self, String> {
        Ok(Self {
            leap_type: self.leap_type.to_aliased(aliases)?,
            path: self.path.clone(),
        })
    }
}
