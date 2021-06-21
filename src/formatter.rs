use crate::{
    leaptypes::{Comment, CommentType, LeapEnum, LeapStruct, LeapType, Name, PropType},
    parser::{commentsparser::parse, parser::Parser, itemposition::ItemPosition},
};

// todo: need assign position info for parsed LeapTypes (pos + len?), assign path to LeapType?
pub fn format(data: &str) -> Result<String, ItemPosition<String>> {
    let types = Parser::parse(data)?;
    let mut comments = parse(data).into_iter().peekable();
    let mut formatted = vec![];
    for next_type in types {
        // loop {
        //     if let Some(p) = comments.peek() {
        //         if p.0 < next_type
        //     } else {
        //         break;
        //     }
        // }
        formatted.push(format_type(&&next_type));
    }
    Ok(formatted.join("\n"))
}

fn format_type(leap_type: &LeapType) -> String {
    match leap_type {
        LeapType::Struct(s) => format_struct(s),
        LeapType::Enum(e) => format_enum(e),
    }
}

fn format_struct(leap_struct: &LeapStruct) -> String {
    let mut lines = vec![];
    lines.push(format!(
        ".struct {}{}",
        leap_struct.name.get(),
        format_type_args(&leap_struct.args)
    ));
    for prop in &leap_struct.props {
        lines.push(format!(
            "    {}: {}",
            prop.name.get(),
            format_prop_type(&prop.prop_type)
        ));
    }
    lines.join("\n")
}
fn format_enum(leap_enum: &LeapEnum) -> String {
    let mut lines = vec![];
    lines.push(format!(
        ".enum {}{}",
        leap_enum.name.get(),
        format_type_args(&leap_enum.args)
    ));
    for variant in &leap_enum.variants {
        lines.push(format!("    {}", format_prop_type(&variant.prop_type)));
    }
    lines.join("\n")
}

fn format_type_args(args: &[Name]) -> String {
    if args.is_empty() {
        "".to_owned()
    } else {
        let mut tokens = vec![];
        for name in args {
            tokens.push(name.get());
        }
        let tokens = tokens.join(" ");
        format!("[{}]", tokens)
    }
}

fn format_prop_type(prop_type: &PropType) -> String {
    match prop_type {
        PropType::Simple(t) => t.name(),
        PropType::List(t) => format!("list[{}]", format_prop_type(t)),
        PropType::TypeArg(n) => n.get().to_owned(),
        PropType::LeapType { name, args } => {
            if args.is_empty() {
                name.get().to_owned()
            } else {
                let args = args
                    .iter()
                    .map(format_prop_type)
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{}[{}]", name.get(), args)
            }
        }
    }
}

fn format_top_comment(comment: &Comment) -> String {
    match comment.comment_type {
        CommentType::Line => format!("/ {}", comment.comment),
        CommentType::LineAndSeparator => format!("/ {}\n", comment.comment),
        _ => panic!("unexpected top comment type"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        assert_eq!(format(".struct    s1").unwrap(), ".struct s1");
        assert_eq!(
            format(".struct    s1 [  a   b   ]").unwrap(),
            ".struct s1[a b]"
        );
        assert_eq!(
            format(".struct    s1  a:   int").unwrap(),
            ".struct s1\n    a: int"
        );
        assert_eq!(format(".enum    e1    s1").unwrap(), ".enum e1\n    s1");
        assert_eq!(
            format(".enum    e1[a   b]    s1  v2[ a   b ]").unwrap(),
            ".enum e1[a b]\n    s1\n    v2[a b]"
        );
    }

    #[test]
    fn test_format_with_comments() {}
}
