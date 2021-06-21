use super::itemposition::ItemPosition;
use crate::leaptypes::{Comment, CommentType};

// comments parser is separte from types parser for simplicity
pub fn parse(data: &str) -> Vec<ItemPosition<Comment>> {
    let mut comments = vec![];
    let mut chars = data.chars().enumerate();
    // let v: String = chars;
    while let Some((i, v)) = chars.next() {
        if v == '/' {
            // ::Line or ::LineAndSeparator comment
            let comment = consume_up_to_line_end(&mut chars);
            comments.push(ItemPosition::new(
                i,
                comment.len() + 1,
                Comment {
                    comment: comment.trim().to_owned(),
                    comment_type: CommentType::Line,
                },
            ))
        } else if v == '\n' {
            // found empty line
            if let Some(c) = comments.last_mut() {
                if c.1.comment_type == CommentType::Line {
                    c.1.comment_type = CommentType::LineAndSeparator;
                }
            }
        } else if !v.is_whitespace() {
            // ::Trail comment
            // skip up to `/` or new line
            while let Some((i, v)) = chars.next() {
                if v == '/' {
                    let comment = consume_up_to_line_end(&mut chars);
                    comments.push(ItemPosition::new(
                        i,
                        comment.len() + 1,
                        Comment {
                            comment: comment.trim().to_owned(),
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
        assert_eq!(comments[0].0.start, 4);
        assert_eq!(comments[0].0.length, 10);
        assert_eq!(comments[0].1.comment, "aaaa");
        assert_eq!(comments[0].1.comment_type, CommentType::Line);
        let comments = parse("    /   aaaa  \n   \nbbb");
        assert_eq!(comments[0].0.start, 4);
        assert_eq!(comments[0].0.length, 10);
        assert_eq!(comments[0].1.comment, "aaaa");
        assert_eq!(comments[0].1.comment_type, CommentType::LineAndSeparator);
        let comments = parse("  text /  aaaa   ");
        assert_eq!(comments[0].0.start, 7);
        assert_eq!(comments[0].0.length, 10);
        assert_eq!(comments[0].1.comment, "aaaa");
        assert_eq!(comments[0].1.comment_type, CommentType::Trail);
    }
}
