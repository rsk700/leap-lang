use super::position::Position;
use crate::leaptypes::{Comment, CommentType};

// comments parser is separte from types parser for simplicity
pub fn parse(data: &str) -> Vec<Position<Comment>> {
    let mut comments = vec![];
    let mut chars = data.chars().enumerate();
    // let v: String = chars;
    while let Some((i, v)) = chars.next() {
        if v == '/' {
            // ::Line or ::LineAndSeparator comment
            // todo: detect ::LineAndSeparator comment
            comments.push(Position(
                i,
                Comment {
                    comment: consume_up_to_line_end(&mut chars).trim().to_owned(),
                    comment_type: CommentType::Line,
                },
            ))
        } else if !v.is_whitespace() {
            // ::Trail comment
            // skip up to `/` or new line
            while let Some((i, v)) = chars.next() {
                if v == '/' {
                    comments.push(Position(
                        i,
                        Comment {
                            comment: consume_up_to_line_end(&mut chars).trim().to_owned(),
                            comment_type: CommentType::Trail,
                        },
                    ))
                } else if v == '\n' {
                    break;
                }
            }
        }
    }
    comments
}

fn consume_up_to_line_end(chars: &mut impl Iterator<Item = (usize, char)>) -> String {
    let mut s = vec![];
    for (_, v) in chars {
        if v == '\n' {
            break;
        }
        s.push(v);
    }
    s.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(parse("aaa\nbbb\nccc").len(), 0);
        let comments = parse("    /   aaaa  ");
        assert_eq!(comments[0].1.comment, "aaaa");
        assert_eq!(comments[0].1.comment_type, CommentType::Line);
        let comments = parse("    /   aaaa  \n   \n");
        assert_eq!(comments[0].1.comment, "aaaa");
        assert_eq!(comments[0].1.comment_type, CommentType::LineAndSeparator);
        let comments = parse("  text /  aaaa   ");
        assert_eq!(comments[0].1.comment, "aaaa");
        assert_eq!(comments[0].1.comment_type, CommentType::Trail);
    }
}
