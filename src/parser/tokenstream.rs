use super::{itemposition::ItemPosition, token::Token};

pub struct TokenStream {
    cursor: usize,
    tokens: Vec<ItemPosition<Token>>,
    end: ItemPosition<Token>,
}

impl TokenStream {
    pub fn new(data: &str) -> Self {
        Self {
            cursor: 0,
            tokens: Self::tokenize(data),
            // end position for return if tokens is empty
            end: ItemPosition::new(0, 0, Token::End),
        }
    }

    pub fn tokenize(data: &str) -> Vec<ItemPosition<Token>> {
        let mut tokens: Vec<ItemPosition<Token>> = Vec::new();
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
                    tokens.push(ItemPosition::new(word_index, word.len(), Token::Word(word)));
                    tokens.push(ItemPosition::new(i, 1, Token::BracketLeft));
                    word = String::new();
                }
                ']' => {
                    tokens.push(ItemPosition::new(word_index, word.len(), Token::Word(word)));
                    tokens.push(ItemPosition::new(i, 1, Token::BracketRight));
                    word = String::new();
                }
                ':' => {
                    tokens.push(ItemPosition::new(word_index, word.len(), Token::Word(word)));
                    tokens.push(ItemPosition::new(i, 1, Token::Colon));
                    word = String::new();
                }
                '/' => {
                    tokens.push(ItemPosition::new(word_index, word.len(), Token::Word(word)));
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
                        tokens.push(ItemPosition::new(word_index, word.len(), Token::Word(word)));
                        word = String::new();
                    }
                }
            }
        }
        tokens.push(ItemPosition::new(word_index, word.len(), Token::Word(word)));
        tokens = tokens
            .into_iter()
            .filter(|t| {
                if let ItemPosition(.., Token::Word(w)) = t {
                    !w.is_empty()
                } else {
                    true
                }
            })
            .map(|t| match &t {
                ItemPosition(.., Token::Word(w)) => match w.as_str() {
                    ".struct" => t.replace(Token::Struct),
                    ".enum" => t.replace(Token::Enum),
                    _ => t,
                },
                _ => t,
            })
            .collect();
        // End token goes right after the last token, in order to show positional
        // information more correctly (eg. ignore trailing spaces)
        let end_index = if let Some(last) = tokens.last() {
            last.0.end()
        } else {
            0
        };
        tokens.push(ItemPosition::new(end_index, 0, Token::End));
        tokens
    }

    pub fn next(&mut self) {
        self.cursor += 1;
    }

    pub fn consume(&mut self) -> &ItemPosition<Token> {
        self.cursor += 1;
        self.get_by_index(self.cursor - 1)
    }

    pub fn get(&self) -> &ItemPosition<Token> {
        self.get_by_index(self.cursor)
    }

    pub fn get_by_index(&self, i: usize) -> &ItemPosition<Token> {
        self.tokens.get(i).unwrap_or_else(|| self.get_end())
    }

    pub fn get_end(&self) -> &ItemPosition<Token> {
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
                ItemPosition::new(0, 7, Token::Struct),
                ItemPosition::new(8, 5, Token::Word("aa-aa".to_owned())),
                ItemPosition::new(13, 0, Token::End)
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
                ItemPosition::new(0, 5, Token::Enum),
                ItemPosition::new(6, 3, Token::Word("aaa".to_owned())),
                ItemPosition::new(9, 0, Token::End)
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
                ItemPosition::new(0, 7, Token::Struct),
                ItemPosition::new(8, 3, Token::Word("aaa".to_owned())),
                ItemPosition::new(11, 1, Token::BracketLeft),
                ItemPosition::new(12, 1, Token::Word("a".to_owned())),
                ItemPosition::new(14, 1, Token::Word("b".to_owned())),
                ItemPosition::new(15, 1, Token::BracketRight),
                ItemPosition::new(21, 1, Token::Word("a".to_owned())),
                ItemPosition::new(22, 1, Token::Colon),
                ItemPosition::new(24, 3, Token::Word("int".to_owned())),
                ItemPosition::new(32, 1, Token::Word("b".to_owned())),
                ItemPosition::new(33, 1, Token::Colon),
                ItemPosition::new(35, 3, Token::Word("str".to_owned())),
                ItemPosition::new(53, 1, Token::Word("c".to_owned())),
                ItemPosition::new(54, 1, Token::Colon),
                ItemPosition::new(56, 4, Token::Word("bool".to_owned())),
                ItemPosition::new(60, 0, Token::End)
            ]
        );
    }
}
