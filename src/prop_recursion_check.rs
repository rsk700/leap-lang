use crate::leaptypes::*;
use std::collections::HashSet;

pub struct PropRecursionCheck<'a> {
    spec: &'a LeapSpec,
    start_name: String,
    visited: HashSet<PropType>,
}

impl<'a> PropRecursionCheck<'a> {
    pub fn is_recursive(spec: &'a LeapSpec, leap_type: &LeapType, prop: &Prop) -> bool {
        let mut check = Self {
            spec,
            start_name: leap_type.name().get().to_owned(),
            visited: HashSet::new(),
        };
        check.is_recursive_check(&prop.prop_type)
    }

    fn is_recursive_check(&mut self, next: &PropType) -> bool {
        if self.start_name == next.name() {
            return true;
        }
        if self.visited.contains(next) {
            return false;
        }
        self.visited.insert(next.clone());
        // get type if it is .struct or .enum
        let next_h = if let Some(h) = self.spec.get_type_by_name(&next.name()) {
            h
        } else {
            return false;
        };
        let next_t = self.spec.apply_args(next_h, &next.args());
        match next_t {
            LeapType::Struct(s) => {
                for Prop { prop_type, .. } in &s.props {
                    if self.is_recursive_check(prop_type) {
                        return true;
                    }
                }
            }
            LeapType::Enum(e) => {
                for Prop { prop_type, .. } in &e.variants {
                    if self.is_recursive_check(prop_type) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{parser::parser::Parser, stdtypes::STD_TYPES};

    #[test]
    fn test_simple1() {
        let spec_text = "
            .struct s1
                a: str
        ";
        let spec = LeapSpec::new(Parser::parse(spec_text).unwrap());
        let h = spec.get_type_by_name("s1").unwrap();
        let t = spec.get_type_ref(h);
        let s = t.unwrap_struct_ref();
        assert!(!PropRecursionCheck::is_recursive(&spec, t, &s.props[0]));
    }

    #[test]
    fn test_simple2() {
        let spec_text = "
            .struct s1
                a: s1
        ";
        let spec = LeapSpec::new(Parser::parse(spec_text).unwrap());
        let h = spec.get_type_by_name("s1").unwrap();
        let t = spec.get_type_ref(h);
        let s = t.unwrap_struct_ref();
        assert!(PropRecursionCheck::is_recursive(&spec, t, &s.props[0]));
    }

    #[test]
    fn test_simple3() {
        let spec_text = "
            .struct s1
                a: e

            .struct s2
                a: float

            .enum e
                s1
                s2
        ";
        let spec = LeapSpec::new(Parser::parse(spec_text).unwrap());
        let h = spec.get_type_by_name("e").unwrap();
        let t = spec.get_type_ref(h);
        let e = t.unwrap_enum_ref();
        assert!(PropRecursionCheck::is_recursive(&spec, t, &e.variants[0]));
        assert!(!PropRecursionCheck::is_recursive(&spec, t, &e.variants[1]));
    }

    #[test]
    fn test_complex() {
        let spec_text = "
            .struct s1
                a: s2[option[s3]]
                b: s4

            .struct s2[t]
                a: t

            .struct s3
                a: s1

            .struct s4
                a: option[s5]

            .struct s5
                a: s4
        ";
        let mut spec = LeapSpec::new(Parser::parse(spec_text).unwrap());
        spec.join(LeapSpec::new(Parser::parse(STD_TYPES).unwrap()));
        let h = spec.get_type_by_name("s1").unwrap();
        let t = spec.get_type_ref(h);
        let s = t.unwrap_struct_ref();
        assert!(PropRecursionCheck::is_recursive(&spec, t, &s.props[0]));
        assert!(!PropRecursionCheck::is_recursive(&spec, t, &s.props[1]));

        let h = spec.get_type_by_name("s4").unwrap();
        let t = spec.get_type_ref(h);
        let s = t.unwrap_struct_ref();
        assert!(PropRecursionCheck::is_recursive(&spec, t, &s.props[0]));

        let h = spec.get_type_by_name("s5").unwrap();
        let t = spec.get_type_ref(h);
        let s = t.unwrap_struct_ref();
        assert!(PropRecursionCheck::is_recursive(&spec, t, &s.props[0]));
    }
}
