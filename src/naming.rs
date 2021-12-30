#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WritingStyle {
    Upper,      // UPPER WRITING STYLE
    Lower,      // lower writing style
    Title,      // Title writing style
    LowerCamel, // lower Camel Writing Style
    UpperCamel, // Upeer Camel Writing Style
}

pub fn get_parts(name: &str) -> Vec<String> {
    name.split('-').map(String::from).collect()
}

pub fn apply_style(style: WritingStyle, separator: &str, names: &[String]) -> String {
    let names: Vec<String> = match style {
        WritingStyle::Upper => names.iter().map(|n| n.to_uppercase()).collect(),
        WritingStyle::Lower => names.iter().map(|n| n.to_lowercase()).collect(),
        WritingStyle::Title => {
            if let Some(first) = names.first() {
                let mut new_names = vec![up_first_char(first)];
                for next_name in &names[1..] {
                    new_names.push(next_name.to_lowercase());
                }
                new_names
            } else {
                vec![]
            }
        }
        WritingStyle::LowerCamel => {
            if let Some(first) = names.first() {
                let mut new_names = vec![first.to_lowercase()];
                for next_name in &names[1..] {
                    new_names.push(up_first_char(next_name));
                }
                new_names
            } else {
                vec![]
            }
        }
        WritingStyle::UpperCamel => names.iter().map(|n| up_first_char(n)).collect(),
    };
    names.join(separator)
}

fn up_first_char(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styles() {
        let text = get_parts("aaa-bbb-ccc");
        assert_eq!("AAA_BBB_CCC", apply_style(WritingStyle::Upper, "_", &text));
        assert_eq!("aaa_bbb_ccc", apply_style(WritingStyle::Lower, "_", &text));
        assert_eq!("Aaa_bbb_ccc", apply_style(WritingStyle::Title, "_", &text));
        assert_eq!(
            "aaa_Bbb_Ccc",
            apply_style(WritingStyle::LowerCamel, "_", &text)
        );
        assert_eq!(
            "Aaa_Bbb_Ccc",
            apply_style(WritingStyle::UpperCamel, "_", &text)
        );
    }
}
