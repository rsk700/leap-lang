use crate::{
    leaptypes::{Comment, CommentType, LeapEnum, LeapStruct, LeapType, Name, PropType},
    parser::{commentsparser, parser::Parser},
};

#[derive(Debug)]
struct Block {
    start: usize,
    next_start: usize,
    trail_indent: usize,
    new_section: bool,
    text: String,
}

pub fn format(data: &str) -> Option<String> {
    let types = Parser::parse(data).ok()?;
    let mut formatted: Vec<Block> = vec![];
    for next_type in &types {
        let mut lines = format_type(next_type);
        if !lines.is_empty() {
            if let Some(last) = formatted.last_mut() {
                last.next_start = lines.last().unwrap().start;
            }
            update_trail_indent(&mut lines);
            formatted.append(&mut lines);
        }
    }
    if let Some(last) = formatted.last_mut() {
        last.next_start = data.len();
    }
    let mut comments = commentsparser::parse(data).into_iter().peekable();
    let mut result = vec![];
    for b in formatted {
        while comments
            .peek()
            .map_or(false, |c| c.position.start < b.start)
        {
            let indent = if b.new_section { 0 } else { 4 };
            result.push(format_comment(&comments.next().unwrap(), indent));
        }
        let has_trail_comment = comments.peek().map_or(false, |c| match c.comment_type {
            CommentType::Trail => b.start < c.position.start && c.position.start < b.next_start,
            _ => false,
        });
        if has_trail_comment {
            let indent = b.trail_indent - b.text.len();
            let mut text = b.text;
            text.push_str(&format_comment(&comments.next().unwrap(), indent));
            result.push(text);
        } else {
            result.push(b.text);
        }
    }
    for c in comments {
        result.push(format_comment(&c, 0));
    }
    // remove repeating separators and separators at the start
    let mut new_result: Vec<String> = vec![];
    for r in result {
        if r.is_empty() {
            // push only if last is not empty and array is not empty
            let last_separator_or_empty = new_result.last().map(|s| s.is_empty()).unwrap_or(true);
            if !last_separator_or_empty {
                new_result.push(r)
            }
        } else {
            new_result.push(r);
        }
    }
    result = new_result;
    // remove separators at the end
    if result.last().map(|s| s.is_empty()).unwrap_or(false) {
        result.pop();
    }
    let mut result = result.join("\n");
    // newline at the end
    if !result.is_empty() {
        result.push('\n');
    }
    Some(result)
}

fn update_trail_indent(lines: &mut [Block]) {
    let max_len = lines.iter().map(|b| b.text.len()).max().unwrap_or(0);
    let indent = (max_len / 4) * 4 + 4;
    for b in lines {
        b.trail_indent = indent;
    }
}

fn format_type(leap_type: &LeapType) -> Vec<Block> {
    let mut lines = vec![];
    let mut type_lines = match leap_type {
        LeapType::Struct(s) => format_struct(s),
        LeapType::Enum(e) => format_enum(e),
    };
    lines.append(&mut type_lines);
    lines
}

fn format_struct(leap_struct: &LeapStruct) -> Vec<Block> {
    let mut lines = vec![];
    let text = format!(
        ".struct {}{}",
        leap_struct.name.get(),
        format_type_args(&leap_struct.args)
    );
    let next_start = if let Some(last) = leap_struct.props.last() {
        last.position.start
    } else {
        leap_struct.position.end()
    };
    lines.push(Block {
        start: leap_struct.position.start,
        next_start,
        trail_indent: 0,
        new_section: true,
        text,
    });
    for i in 0..leap_struct.props.len() {
        let prop = &leap_struct.props[i];
        let next_prop = leap_struct.props.get(i + 1);
        let text = format!(
            "    {}: {}",
            prop.name.get(),
            format_prop_type(&prop.prop_type)
        );
        let next_start = if let Some(next) = next_prop {
            next.position.start
        } else {
            prop.position.end()
        };
        lines.push(Block {
            start: prop.position.start,
            next_start,
            trail_indent: 0,
            new_section: false,
            text,
        })
    }
    lines
}

fn format_enum(leap_enum: &LeapEnum) -> Vec<Block> {
    let mut lines = vec![];
    let text = format!(
        ".enum {}{}",
        leap_enum.name.get(),
        format_type_args(&leap_enum.args)
    );
    let next_start = if let Some(last) = leap_enum.variants.last() {
        last.position.start
    } else {
        leap_enum.position.end()
    };
    lines.push(Block {
        start: leap_enum.position.start,
        next_start,
        trail_indent: 0,
        new_section: true,
        text,
    });
    for i in 0..leap_enum.variants.len() {
        let variant = &leap_enum.variants[i];
        let next_var = leap_enum.variants.get(i + 1);
        let text = format!("    {}", format_prop_type(&variant.prop_type));
        let next_start = if let Some(next) = next_var {
            next.position.start
        } else {
            variant.position.end()
        };
        lines.push(Block {
            start: variant.position.start,
            next_start,
            trail_indent: 0,
            new_section: false,
            text,
        })
    }
    lines
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

fn format_comment(comment: &Comment, indent: usize) -> String {
    let indent = String::from_utf8(vec![b' '; indent]).unwrap();
    match comment.comment_type {
        CommentType::Line | CommentType::Trail => format!("{}/ {}", indent, comment.comment),
        CommentType::Separator => "".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_trail_indent() {
        let mut blocks = vec![Block {
            start: 0,
            next_start: 0,
            trail_indent: 0,
            new_section: false,
            text: "aaaa".to_owned(),
        }];
        update_trail_indent(&mut blocks);
        assert_eq!(blocks[0].trail_indent, 8);
        let mut blocks = vec![Block {
            start: 0,
            next_start: 0,
            trail_indent: 0,
            new_section: false,
            text: "aaaaaaa".to_owned(),
        }];
        update_trail_indent(&mut blocks);
        assert_eq!(blocks[0].trail_indent, 8);
    }

    #[test]
    fn test_format() {
        assert_eq!(format(".struct    s1").unwrap(), ".struct s1\n");
        assert_eq!(
            format(".struct    s1 [  a   b   ]").unwrap(),
            ".struct s1[a b]\n"
        );
        assert_eq!(
            format(".struct    s1  a:   int").unwrap(),
            ".struct s1\n    a: int\n"
        );
        assert_eq!(format(".enum    e1    s1").unwrap(), ".enum e1\n    s1\n");
        assert_eq!(
            format(".enum    e1[a   b]    s1  v2[ a   b ]").unwrap(),
            ".enum e1[a b]\n    s1\n    v2[a b]\n"
        );
    }

    #[test]
    fn test_format_with_comments() {
        assert_eq!(format("").unwrap(), "");
        assert_eq!(
            format("/ text\n.struct    s1").unwrap(),
            "/ text\n.struct s1\n"
        );
        assert_eq!(
            format("/ text\n\n.struct    s1").unwrap(),
            "/ text\n\n.struct s1\n"
        );
        assert_eq!(
            format("\n\n\n\n.struct  \n\n\n\n  s1\n\n\n\n").unwrap(),
            ".struct s1\n"
        );
        assert_eq!(
            format("/ text\n\n\n.struct    s1").unwrap(),
            "/ text\n\n.struct s1\n"
        );
        assert_eq!(
            format("/ text     \n\n\n.struct    s1").unwrap(),
            "/ text\n\n.struct s1\n"
        );

        assert_eq!(format(".struct s1 / text").unwrap(), ".struct s1  / text\n");
        assert_eq!(format(".enum s1 / text").unwrap(), ".enum s1    / text\n");

        assert_eq!(
            format(".struct s1\n/ text").unwrap(),
            ".struct s1\n/ text\n"
        );
        assert_eq!(format(".enum s1\n/ text").unwrap(), ".enum s1\n/ text\n");

        assert_eq!(
            format(".struct s1\nv: int / text").unwrap(),
            ".struct s1\n    v: int  / text\n"
        );
        assert_eq!(
            format(".enum s1\nval / text").unwrap(),
            ".enum s1\n    val     / text\n"
        );

        assert_eq!(
            format(".struct s1\n/ text\nv: int").unwrap(),
            ".struct s1\n    / text\n    v: int\n"
        );
        assert_eq!(
            format(".enum aaaaaa\n/ text\nval").unwrap(),
            ".enum aaaaaa\n    / text\n    val\n"
        );

        assert_eq!(
            format(".struct s1\n/ text\n\n\n/ text\nv: int").unwrap(),
            ".struct s1\n    / text\n\n    / text\n    v: int\n"
        );
    }

    #[test]
    fn test_format_complex() {
        let formatted = format(
            "
        / text1 text
        / tttt2

        /text3

        .struct some-my-struct / text4
        /text5

        /text6
        /text7
        v1: list[int] /text8
        v2: list[list[int]]

        /text9
        /text10

        .enum value-enum

        / test11

        val1
        val2

        / text12
        ",
        )
        .unwrap();
        let expected = "/ text1 text
/ tttt2

/ text3

.struct some-my-struct  / text4
    / text5

    / text6
    / text7
    v1: list[int]       / text8
    v2: list[list[int]]

/ text9
/ text10

.enum value-enum

    / test11

    val1
    val2

/ text12
";
        assert_eq!(formatted, expected);
        // if format formatted, result should be same
        assert_eq!(format(&formatted).unwrap(), expected);
    }
}
