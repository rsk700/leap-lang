use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

use crate::{
    leaptypes::{Comment, CommentType},
    parser::position::Position,
};

// comments parser is separte from types parser for simplicity
pub fn parse(data: &str) -> Vec<Comment> {
    let mut comments = vec![];
    let mut chars = data.chars().enumerate().peekable();
    // let v: String = chars;
    while let Some((i, v)) = chars.next() {
        if v == '/' {
            // ::Line comment
            let (comment, length) = parse_comment(&mut chars);
            comments.push(Comment {
                comment,
                comment_type: CommentType::Line,
                position: Position::new(i, length),
            });
        } else if v.is_whitespace() {
            let mut v = v;
            while let Some((_, next_v)) = chars.peek() {
                if v == '\n' {
                    break;
                };
                if !next_v.is_whitespace() {
                    // not an empty line
                    break;
                }
                v = if let Some((_, new_v)) = chars.next() {
                    new_v
                } else {
                    break;
                };
            }
            if v == '\n' {
                // found empty line
                // ::Separator
                comments.push(Comment {
                    comment: String::new(),
                    comment_type: CommentType::Separator,
                    position: Position::new(i, 0),
                });
            }
        } else if !v.is_whitespace() {
            // ::Trail comment
            // skip up to `/` or new line
            for (i, v) in &mut chars {
                match v {
                    '/' => {
                        let (comment, length) = parse_comment(&mut chars);
                        comments.push(Comment {
                            comment,
                            comment_type: CommentType::Trail,
                            position: Position::new(i, length),
                        });
                        break;
                    }
                    '\n' => break,
                    _ => {}
                };
            }
        }
    }
    comments
}

fn parse_comment(chars: &mut Peekable<Enumerate<Chars>>) -> (String, usize) {
    let mut comment_mark_len = 1;
    while chars.peek().map(|(_, c)| *c == '-').unwrap_or(false) {
        chars.next();
        comment_mark_len += 1;
        if comment_mark_len == 3 {
            // prefix is `/--`
            break;
        }
    }
    let comment = consume_up_to_line_end(chars);
    (comment.trim().to_owned(), comment_mark_len + comment.len())
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
        let comments = parse("    /--   aaaa  ");
        assert_eq!(comments[0].position.start, 4);
        assert_eq!(comments[0].position.length, 12);
        assert_eq!(comments[0].comment, "aaaa");
        assert_eq!(comments[0].comment_type, CommentType::Line);
        let comments = parse("    /-   aaaa  ");
        assert_eq!(comments[0].position.start, 4);
        assert_eq!(comments[0].position.length, 11);
        assert_eq!(comments[0].comment, "aaaa");
        assert_eq!(comments[0].comment_type, CommentType::Line);
        let comments = parse("    /   aaaa  ");
        assert_eq!(comments[0].position.start, 4);
        assert_eq!(comments[0].position.length, 10);
        assert_eq!(comments[0].comment, "aaaa");
        assert_eq!(comments[0].comment_type, CommentType::Line);
        let comments = parse("    /---   aaaa  ");
        assert_eq!(comments[0].position.start, 4);
        assert_eq!(comments[0].position.length, 13);
        assert_eq!(comments[0].comment, "-   aaaa");
        assert_eq!(comments[0].comment_type, CommentType::Line);
        let comments = parse("    /--   aaaa  \n   \nbbb");
        assert_eq!(comments[0].position.start, 4);
        assert_eq!(comments[0].position.length, 12);
        assert_eq!(comments[0].comment, "aaaa");
        assert_eq!(comments[0].comment_type, CommentType::Line);
        assert_eq!(comments[1].position.start, 17);
        assert_eq!(comments[1].position.length, 0);
        assert_eq!(comments[1].comment_type, CommentType::Separator);
        let comments = parse("  text /--  aaaa   ");
        assert_eq!(comments[0].position.start, 7);
        assert_eq!(comments[0].position.length, 12);
        assert_eq!(comments[0].comment, "aaaa");
        assert_eq!(comments[0].comment_type, CommentType::Trail);
        let comments = parse(".struct some-my-struct /-- text4\n\n/text5\n\nv: int");
        assert_eq!(comments[1].comment_type, CommentType::Separator);
        assert_eq!(comments[0].comment_type, CommentType::Trail);
        assert_eq!(comments[1].comment_type, CommentType::Separator);
        let comments = parse(".struct some-my-struct\n\n\n\n\n");
        assert_eq!(comments.len(), 4);
        let comments = parse("\n\n\n\n\n.struct some-my-struct");
        assert_eq!(comments.len(), 5);
        let comments = parse(".enum my-enum\n\n\n\n\n.struct some-my-struct");
        assert_eq!(comments.len(), 4);
        let comments = parse("/-- text");
        assert_eq!(comments[0].comment, "text");
    }
}
