#[derive(PartialEq, Debug)]
pub enum Token {
    Word(String),
    // ".struct"
    Struct,
    // ".enum"
    Enum,
    // "["
    BracketLeft,
    // "]"
    BracketRight,
    // ":"
    Colon,
    End,
}