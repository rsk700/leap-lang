use crate::leaptypes::Name;
use super::{position::Position, proptypesimple::PropTypeSimple};

#[derive(Debug)]
pub struct PropSimple {
    pub name: Name,
    pub prop_type_simple: PropTypeSimple,
    pub position: Position,
}
