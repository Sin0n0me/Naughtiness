pub mod identifier;
pub mod lexer;

#[cfg(test)]
mod tests;

#[derive(Debug, Eq, PartialEq)]
pub enum LiteralKind {
    Unknown,
    BinLiteral,
    OctLiteral,
    DecLiteral,
    HexLiteral,
    FloatLiteral(bool), // is_exponent: bool
    CharacterLiteral,
    StringLiteral,
    RawStringLiteral,
    ByteLiteral,
    ByteStringLiteral,
    RawByteStringLiteral,
    CStringLiteral,
    RawCStringLiteral,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TokenKind {
    Unkown,
    Identifier(String),
    Prefix,
    Literal(LiteralKind),
    Comment,
    WhiteSpace,

    LeftParenthesis,  // (
    RightParenthesis, // )
    LeftBrackets,     // [
    RightBrackets,    // ]
    LeftBrace,        // {
    RightBrace,       // }

    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    Equal,       // =
    Caret,       // ^
    Not,         // !
    And,         // &
    Or,          // |
    GreaterThan, //  >
    LessThan,    // <
    At,          // @
    Dot,         // .
    Comma,       // ,
    Colon,       // :
    Semicolon,   // ;
    Underscore,  // _
    Pound,       // #
    Dollar,      // $
    Question,    // ?
    Tilde,       // ~

    Eof,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Token {
    pub token_kind: TokenKind,
    pub token: String,
}

impl Token {
    pub fn new(token_kind: TokenKind, token: &str) -> Self {
        Self {
            token_kind,
            token: token.to_string(),
        }
    }
}
