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
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    XorAssign,
    OrAssign,
    AndAssign,
    LeftShiftAssign,
    RightShihtAssign,

    ConditionCompare,
    ConditionNotCompare,
    ConditionAnd,
    ConditionOr,
    ConditionLessThan,
    ConditionGreaterThan,
    ConditionLessThanEquel,
    ConditionGreaterThanEqual,
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
pub enum Parenthesis {
    Parenthesis, // ()
    Brackets,    // []
    Brace,       // {}
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Literal {
    pub literal_kind: LiteralKind,
    pub prefix: String,
    pub symbol: String,
    pub suffix: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum BitWidth {
    Bit8,
    Bit16,
    Bit32,
    Bit64,
    Bit128,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Types {
    Integer {
        bit_width: BitWidth,
        is_unsigned: bool,
    },
    Float(BitWidth),
    Pointer,
    Array,
    Struct(Vec<Types>),
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
    LeftParenthesis(Parenthesis),
    RightParenthesis(Parenthesis),

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
