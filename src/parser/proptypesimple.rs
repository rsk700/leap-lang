use crate::leaptypes::{Name, ValueType, SimpleType};

use super::position::Position;

#[derive(Debug)]
pub struct PropTypeSimple {
    pub name: String,
    pub args: Vec<PropTypeSimple>,
    pub position: Position,
}

impl PropTypeSimple {
    pub fn try_into_prop_type(mut self, type_args: &[Name]) -> Result<ValueType, String> {
        match self.name.as_str() {
            "str" => {
                if self.args.is_empty() {
                    Ok(ValueType::Simple(SimpleType::String))
                } else {
                    Err("String type should not have arguments".to_owned())
                }
            }
            "int" => {
                if self.args.is_empty() {
                    Ok(ValueType::Simple(SimpleType::Integer))
                } else {
                    Err("Integer type should not have arguments".to_owned())
                }
            }
            "float" => {
                if self.args.is_empty() {
                    Ok(ValueType::Simple(SimpleType::Float))
                } else {
                    Err("Float type should not have arguments".to_owned())
                }
            }
            "bool" => {
                if self.args.is_empty() {
                    Ok(ValueType::Simple(SimpleType::Boolean))
                } else {
                    Err("Boolean type should not have arguments".to_owned())
                }
            }
            "list" => {
                if self.args.len() == 1 {
                    let list_element = self.args.remove(0).try_into_prop_type(type_args)?;
                    Ok(ValueType::List(Box::new(list_element)))
                } else {
                    Err("List should have exactly one argument".to_owned())
                }
            }
            name => {
                let name = Name::new(name.to_owned(), self.position)?;
                if type_args.contains(&name) {
                    if self.args.is_empty() {
                        Ok(ValueType::TypeArg(name))
                    } else {
                        Err("Type argument can't have arguments".to_owned())
                    }
                } else {
                    let args = self
                        .args
                        .into_iter()
                        .map(|a| a.try_into_prop_type(type_args))
                        .collect::<Result<_, _>>()?;
                    Ok(ValueType::LeapType { name, args })
                }
            }
        }
    }
}
