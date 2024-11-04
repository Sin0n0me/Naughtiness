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

    BinaryOperator,

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

impl Token {
    pub fn is_operator(&self) -> bool {
        is_operator(self)
    }
}
pub fn is_operator(token: &Token) -> bool {
    matches!(
        token,
        Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Percent
            | Token::Caret
            | Token::Not
            | Token::And
            | Token::Or
            | Token::AndAnd
            | Token::OrOr
            | Token::LeftShift
            | Token::RightShift
            | Token::PlusEqual
            | Token::MinusEqual
            | Token::StarEqual
            | Token::SlashEqual
            | Token::PercentEqual
            | Token::CaretEqual
            | Token::AndEqual
            | Token::OrEqual
            | Token::LeftShiftEqual
            | Token::RightShiftEqual
            | Token::Equal
            | Token::EqualEqual
            | Token::NotEqual
            | Token::GreaterThan
            | Token::LessThan
            | Token::GreaterThanOrEqual
            | Token::LessThanOrEqual
            | Token::At
            | Token::Underscore
            | Token::Dot
            | Token::DotDot
            | Token::DotDotDot
            | Token::DotDotEqual
            | Token::Comma
            | Token::Semicolon
            | Token::Colon
            | Token::PathSeparater
            | Token::RightAllow
            | Token::FatAllow
            | Token::LeftAllow
            | Token::Pound
            | Token::Dollar
            | Token::Question
            | Token::Tilde
    )
}
