use super::{position::Position, token::Token};

pub struct TokenStream {
    cursor: usize,
    tokens: Vec<Position<Token>>,
    end: Position<Token>,
}

impl TokenStream {
    pub fn new(data: &str) -> Self {
        Self {
            cursor: 0,
            tokens: Self::tokenize(data),
            // end position for return if tokens is empty
            end: Position(0, Token::End),
        }
    }

    pub fn tokenize(data: &str) -> Vec<Position<Token>> {
        let mut tokens: Vec<Position<Token>> = Vec::new();
        let mut word = String::new();
        let mut word_index = 0;
        let mut is_comment = false;
        for (i, v) in data.chars().enumerate() {
            if is_comment {
                if v == '\n' {
                    is_comment = false;
                }
                continue;
            }
            match v {
                '[' => {
                    tokens.push(Position(word_index, Token::Word(word)));
                    tokens.push(Position(i, Token::BracketLeft));
                    word = String::new();
                }
                ']' => {
                    tokens.push(Position(word_index, Token::Word(word)));
                    tokens.push(Position(i, Token::BracketRight));
                    word = String::new();
                }
                ':' => {
                    tokens.push(Position(word_index, Token::Word(word)));
                    tokens.push(Position(i, Token::Colon));
                    word = String::new();
                }
                '/' => {
                    tokens.push(Position(word_index, Token::Word(word)));
                    word = String::new();
                    is_comment = true;
                }
                _ => {
                    // todo: word should start with letter (check in Name?)
                    if v.is_alphanumeric() || v == '-' || v == '.' {
                        if word.is_empty() {
                            word_index = i;
                        }
                        word.push(v)
                    } else {
                        tokens.push(Position(word_index, Token::Word(word)));
                        word = String::new();
                    }
                }
            }
        }
        tokens.push(Position(word_index, Token::Word(word)));
        // Adding End token, with position information, because position is not
        // possible to figure it out from previous tokens
        tokens.push(Position(data.len(), Token::End));
        tokens
            .into_iter()
            .filter(|t| {
                if let Position(.., Token::Word(w)) = t {
                    !w.is_empty()
                } else {
                    true
                }
            })
            .map(|t| match &t {
                Position(p, Token::Word(w)) => match w.as_str() {
                    ".struct" => Position(*p, Token::Struct),
                    ".enum" => Position(*p, Token::Enum),
                    _ => t,
                },
                _ => t,
            })
            .collect()
    }

    pub fn next(&mut self) {
        self.cursor += 1;
    }

    pub fn consume(&mut self) -> &Position<Token> {
        self.cursor += 1;
        self.get_by_index(self.cursor - 1)
    }

    pub fn get(&self) -> &Position<Token> {
        self.get_by_index(self.cursor)
    }

    pub fn get_by_index(&self, i: usize) -> &Position<Token> {
        self.tokens.get(i).unwrap_or_else(|| self.get_end())
    }

    pub fn get_end(&self) -> &Position<Token> {
        match self.tokens.len() {
            0 => &self.end,
            _ => &self.tokens[self.tokens.len() - 1],
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_tokenizer_empty_struct() {
        let text = ".struct aa-aa";
        let tokens = TokenStream::tokenize(text);
        assert_eq!(
            tokens,
            vec![
                Position(0, Token::Struct),
                Position(8, Token::Word("aa-aa".to_owned())),
                Position(13, Token::End)
            ]
        );
    }

    #[test]
    fn test_tokenizer_empty_enum() {
        let text = ".enum aaa";
        let tokens = TokenStream::tokenize(text);
        assert_eq!(
            tokens,
            vec![
                Position(0, Token::Enum),
                Position(6, Token::Word("aaa".to_owned())),
                Position(9, Token::End)
            ]
        );
    }

    #[test]
    fn test_tokenizer_struct() {
        let text = ".struct aaa[a b]\n    a: int\n    b: str\n/ comment\n    c: bool";
        let tokens = TokenStream::tokenize(text);
        assert_eq!(
            tokens,
            vec![
                Position(0, Token::Struct),
                Position(8, Token::Word("aaa".to_owned())),
                Position(11, Token::BracketLeft),
                Position(12, Token::Word("a".to_owned())),
                Position(14, Token::Word("b".to_owned())),
                Position(15, Token::BracketRight),
                Position(21, Token::Word("a".to_owned())),
                Position(22, Token::Colon),
                Position(24, Token::Word("int".to_owned())),
                Position(32, Token::Word("b".to_owned())),
                Position(33, Token::Colon),
                Position(35, Token::Word("str".to_owned())),
                Position(53, Token::Word("c".to_owned())),
                Position(54, Token::Colon),
                Position(56, Token::Word("bool".to_owned())),
                Position(60, Token::End)
            ]
        );
    }
}
