pub fn get_parts(name: &str) -> Vec<String> {
    name.split('-').map(String::from).collect()
}

pub fn snake_case(names: &[String]) -> String {
    names.join("_")
}

pub fn upper_camel_case(names: &[String]) -> String {
    names
        .iter()
        .map(|n| n.chars())
        .map(|mut c| match c.next() {
            Some(next_c) => next_c.to_uppercase().chain(c).collect(),
            None => String::new(),
        })
        .collect::<Vec<_>>()
        .join("")
}

pub fn camel_case(names: &[String]) -> String {
    if let Some(first) = names.get(0) {
        format!("{}{}", first, upper_camel_case(&names[1..]))
    } else {
        String::new()
    }
}

pub fn joined(names: &[String]) -> String {
    names.join("")
}

pub fn uppercase_joined(names: &[String]) -> String {
    let names = names.iter().map(|v|v.to_uppercase()).collect::<Vec<_>>();
    joined(&names)
}
