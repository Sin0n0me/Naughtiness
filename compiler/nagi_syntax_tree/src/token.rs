use crate::keywords::Keyword;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum BinaryOperator {
    Equal,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Xor,
    Or,
    And,
    LeftShift,
    RightShiht,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum LiteralKind {
    Bool(bool),
    Byte,
    Char,
    Integer,
    Float,
    Str,
    StrRaw,
    ByteStr,
    ByteStrRaw,
    CStr,
    CStrRaw,
    Error,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum LeftParenthesis {
    Parenthesis, // (
    Brackets,    // [
    Brace,       // {
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum RightParenthesis {
    Parenthesis, // )
    Brackets,    // ]
    Brace,       // }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Literal {
    pub literal_kind: LiteralKind,
    pub prefix: String,
    pub symbol: String,
    pub suffix: String,
}

impl Literal {
    pub fn new(literal_kind: LiteralKind, symbol: &str) -> Self {
        Self {
            literal_kind,
            prefix: "".to_string(),
            symbol: symbol.to_string(),
            suffix: "".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Token {
    // literal
    Literal(Literal),
    Identifier(String),
    Keyword(Keyword),

    //
    LeftParenthesis(LeftParenthesis),
    RightParenthesis(RightParenthesis),

    // operator
    Plus,               // +
    Minus,              // -
    Star,               // *
    Slash,              // /
    Percent,            // %
    Caret,              // ^
    Not,                // !
    And,                // &
    Or,                 // |
    AndAnd,             // &&
    OrOr,               // ||
    LeftShift,          // <<
    RightShift,         // >>
    PlusEqual,          // +=
    MinusEqual,         // -=
    StarEqual,          // *=
    SlashEqual,         // /=
    PercentEqual,       // %=
    CaretEqual,         // ^=
    AndEqual,           // &=
    OrEqual,            // |=
    LeftShiftEqual,     // <<=
    RightShiftEqual,    // >>=
    Equal,              // =
    EqualEqual,         // ==
    NotEqual,           // !=
    GreaterThan,        // >
    LessThan,           // <
    GreaterThanOrEqual, // >=
    LessThanOrEqual,    // <=
    At,                 // @
    Underscore,         // _
    Dot,                // .
    DotDot,             // ..
    DotDotDot,          // ...
    DotDotEqual,        // ..=
    Comma,              // ,
    Semicolon,          // ;
    Colon,              // :
    PathSeparater,      // ::
    RightAllow,         // ->
    FatAllow,           // =>
    LeftAllow,          // <-
    Pound,              // #
    Dollar,             // $
    Question,           // ?
    Tilde,              // ~

    Eof,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub enum Rarity {
    Ur,
    Sr,
    Nr,
    Let,
}
