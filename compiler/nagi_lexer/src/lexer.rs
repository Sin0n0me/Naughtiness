use crate::identifier::{is_identifier_continue, is_identifier_start};
use crate::{LiteralKind, Token, TokenKind};

pub struct Lexer {
    code: Vec<char>,
    token_buffer: String,
    position: usize,
}

impl Lexer {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.chars().collect(),
            token_buffer: "".to_string(),
            position: 0,
        }
    }

    // tokenize

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut token_list = Vec::<Token>::new();
        while let Some(token) = self.next_token() {
            println!("Token: {:?}", token);
            token_list.push(token);
        }

        token_list
    }

    pub fn next_token(&mut self) -> Option<Token> {
        let token_kind = match self.get()? {
            '0'..='9' => {
                let literal = self.number();
                match literal {
                    LiteralKind::BinLiteral
                    | LiteralKind::OctLiteral
                    | LiteralKind::DecLiteral
                    | LiteralKind::HexLiteral => {
                        self.eat_suffix();
                        TokenKind::Literal(literal)
                    }
                    LiteralKind::FloatLiteral(is_exponent) => {
                        if !is_exponent {
                            self.eat_suffix();
                        }

                        TokenKind::Literal(literal)
                    }
                    _ => TokenKind::Unkown,
                }
            }
            '/' => match self.get_next()? {
                '/' => self.line_comment(),
                '*' => self.block_comment(),
                _ => TokenKind::Slash,
            },
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '%' => TokenKind::Percent,
            '=' => TokenKind::Equal,
            '^' => TokenKind::Caret,
            '!' => TokenKind::Not,
            '&' => TokenKind::And,
            '|' => TokenKind::Or,
            '>' => TokenKind::GreaterThan,
            '<' => TokenKind::LessThan,
            '@' => TokenKind::At,
            '.' => TokenKind::Dot,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            ';' => TokenKind::Semicolon,
            '#' => TokenKind::Pound,
            '$' => TokenKind::Dollar,
            '?' => TokenKind::Question,
            '~' => TokenKind::Tilde,
            '(' => TokenKind::LeftParenthesis,
            ')' => TokenKind::RightParenthesis,
            '[' => TokenKind::LeftBrackets,
            ']' => TokenKind::RightBrackets,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,

            _ if self.is_identifier_start() => self.identifier_or_unknown(),
            _ if self.is_white_space() => self.white_space(),

            _ => TokenKind::Unkown,
        };
        if !matches!(
            token_kind,
            TokenKind::Identifier(_)
                | TokenKind::Comment
                | TokenKind::Literal(_)
                | TokenKind::WhiteSpace
        ) {
            self.push_char();
        }

        Some(self.new_token(token_kind))
    }

    // Suffix

    // 現在のトークンがSuffix
    fn eat_suffix(&mut self) -> bool {
        self.eat_identifier_or_keyword()
    }

    fn is_suffix_no_e(&self) -> bool {
        let c = self.token_buffer.chars().nth(0).unwrap();
        !matches!(c, 'e' | 'E')
    }

    //

    fn identifier_or_unknown(&mut self) -> TokenKind {
        if !self.is_identifier_start() {
            return TokenKind::Unkown;
        }

        if self.get().unwrap() == '_' {
            let Some(next) = self.get_next() else {
                return TokenKind::Underscore;
            };
            if !is_identifier_continue(next) {
                return TokenKind::Underscore;
            }
        }

        if !self.eat_identifier_or_keyword() {
            return TokenKind::Unkown;
        }

        TokenKind::Identifier(self.token_buffer.clone())
    }

    // Identifier or Keyword

    // IdentifierOrKeyword     ::= IdentifierStart (IdentifierContinue)* | _ (IdentifierContinue)+
    // RawIdentifier           ::= r# IdentifierOrKeyword Except crate, self, super, Self
    // IdentifierStart         ::= https://util.unicode.org/UnicodeJsps/list-unicodeset.jsp?a=%5B%3AXID_Start%3A%5D&abb=on&g=&i=
    // IdentifierContinue      ::= https://util.unicode.org/UnicodeJsps/list-unicodeset.jsp?a=%5B%3AXID_Continue%3A%5D&abb=on&g=&i=
    // NonkeywordRawIdentifier ::= IdentifierOrKeyword Except a strict or reserved keyword
    // Identifier              ::= NonkeywordRawIdentifier | RawIdentifier
    fn eat_identifier_or_keyword(&mut self) -> bool {
        let mut has_identifier = false;
        if self.is_identifier_start() {
            self.push_char(); // push IdentifierStart
            has_identifier = true;
        } else if self.is_same('_') {
            self.push_char(); // push _
        } else {
            return false;
        }

        while self.is_identifier_continue() {
            self.push_char();
            has_identifier = true;
        }

        has_identifier
    }

    fn is_identifier_start(&mut self) -> bool {
        let Some(c) = self.get() else {
            return false;
        };

        c == '_' || is_identifier_start(c)
    }

    fn is_identifier_continue(&mut self) -> bool {
        let Some(c) = self.get() else {
            return false;
        };

        is_identifier_continue(c)
    }

    //
    fn white_space(&mut self) -> TokenKind {
        while self.is_white_space() {
            self.push_char();
        }

        TokenKind::WhiteSpace
    }

    fn is_white_space(&self) -> bool {
        let Some(c) = self.get() else {
            return false;
        };

        matches!(c, ' ' | '\t' | '\n')
    }

    // Number
    fn number(&mut self) -> LiteralKind {
        // IntgerLiteral ::= (BinLiteral | OctLiteral | DecLiteral | HexLiteral) (SuffixNoE)?
        let Some(first) = self.get() else {
            return LiteralKind::Unknown;
        };

        let literal_kind = match first {
            '0' => {
                // 2文字目でプレフィックスか判断
                let Some(second) = self.get_next() else {
                    self.push_char(); // push 0
                    return LiteralKind::DecLiteral; // only 0
                };

                match second {
                    'b' => {
                        self.push_char(); // push 0
                        self.push_char(); // push b
                        LiteralKind::BinLiteral
                    }
                    'o' => {
                        self.push_char(); // push 0
                        self.push_char(); // push o
                        LiteralKind::OctLiteral
                    }
                    'x' => {
                        self.push_char(); // push 0
                        self.push_char(); // push x
                        LiteralKind::HexLiteral
                    }
                    '0'..='9' | '_' => LiteralKind::DecLiteral,
                    '.' => LiteralKind::DecLiteral, // 後続で処理するためにこの時点では10進数として扱う
                    _ if is_identifier_start(second) => LiteralKind::DecLiteral,
                    _ => return LiteralKind::Unknown,
                }
            }
            _ => LiteralKind::DecLiteral,
        };
        if !match literal_kind {
            LiteralKind::BinLiteral => self.eat_bin_digit(), // BinLiteral ::= 0b (BinDigit | _)* BinDigit (BinDigit | _)*
            LiteralKind::OctLiteral => self.eat_oct_digit(), // OctLiteral ::= 0o (OctDigit | _)* OctDigit (OctDigit | _)*
            LiteralKind::DecLiteral => self.eat_dec_digit(), // DecLiteral ::= DecDigit (DecDigit | _)*
            LiteralKind::HexLiteral => self.eat_hex_digit(), // HexLiteral ::= 0x (HexDigit | _)* HexDigit (HexDigit | _)*
            _ => false,
        } {
            return LiteralKind::Unknown;
        }

        // 2進数や8進数で使用できない数字が使用されているかチェック
        if self.eat_dec_digit() {
            return LiteralKind::Unknown;
        }

        // 10進数のみfloatになる可能性があるので後続の処理へ
        if !matches!(literal_kind, LiteralKind::DecLiteral) {
            return literal_kind;
        }

        // DecLiteralのみ以下処理(float判定)へ

        // FloatLiteral ::= DecLiteral . |
        //                  DecLiteral . DecLiteral (SuffixNoE)? |
        //                  DecLiteral (. DecLiteral)? FloatExponent (Suffix)?

        // .
        let Some(dot) = self.get() else {
            return LiteralKind::DecLiteral;
        };
        match dot {
            '.' => {
                self.push_char(); // push .

                //  DecLiteral
                if !self.eat_dec_digit() {
                    // 1..2, 12.hoge() のようなパターンを弾く
                    // 0. のような場合
                    let Some(second) = self.get_next() else {
                        return LiteralKind::FloatLiteral(false);
                    };
                    if !second.is_ascii_digit() {
                        return LiteralKind::Unknown;
                    }

                    return LiteralKind::Unknown;
                }

                // FloatExponent
                LiteralKind::FloatLiteral(self.eat_float_exponent())
            }
            'e' | 'E' => {
                // 12E
                if self.eat_float_exponent() {
                    LiteralKind::FloatLiteral(true)
                } else {
                    LiteralKind::Unknown
                }
            }
            _ => {
                //  DecLiteral FloatExponent (Suffix)?
                if self.eat_float_exponent() {
                    LiteralKind::FloatLiteral(true)
                } else {
                    LiteralKind::DecLiteral
                }
            }
        }
    }

    // (BinDigit | _)* BinDigit (BinDigit | _)*
    fn eat_bin_digit(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            let Some(c) = self.get() else {
                break;
            };

            match c {
                '_' => self.push_char(),
                _ if self.is_bin_digit() => {
                    self.push_char();
                    has_digits = true;
                }
                _ => break,
            };
        }

        has_digits
    }

    // (OctDigit | _)* OctDigit (OctDigit | _)*
    fn eat_oct_digit(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            let Some(c) = self.get() else {
                break;
            };

            match c {
                '_' => self.push_char(),
                _ if self.is_oct_digit() => {
                    self.push_char();
                    has_digits = true;
                }
                _ => break,
            };
        }

        has_digits
    }

    // DecDigit (DecDigit | _)*
    fn eat_dec_digit(&mut self) -> bool {
        if !self.is_dec_digit() {
            return false;
        }
        self.push_char();

        loop {
            let Some(c) = self.get() else {
                break;
            };

            match c {
                '_' => self.push_char(),
                _ if self.is_dec_digit() => self.push_char(),
                _ => break,
            };
        }

        true
    }

    // (HexDigit | _)* HexDigit (HexDigit | _)*
    fn eat_hex_digit(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            let Some(c) = self.get() else {
                break;
            };

            match c {
                '_' => self.push_char(),
                _ if self.is_hex_digit() => {
                    self.push_char();
                    has_digits = true;
                }
                _ => break,
            };
        }

        has_digits
    }

    fn is_bin_digit(&self) -> bool {
        let Some(c) = self.get() else {
            return false;
        };
        matches!(c, '0'..='1')
    }

    fn is_oct_digit(&self) -> bool {
        let Some(c) = self.get() else {
            return false;
        };
        matches!(c, '0'..='7')
    }

    fn is_dec_digit(&self) -> bool {
        let Some(c) = self.get() else {
            return false;
        };
        c.is_ascii_digit()
    }

    fn is_hex_digit(&self) -> bool {
        let Some(c) = self.get() else {
            return false;
        };
        c.is_ascii_hexdigit()
    }

    // float

    // FloatExponent := [eE] (+ | -)? (DecDigit | _)* DecDigit (DecDigit | _)*
    fn eat_float_exponent(&mut self) -> bool {
        // [eE]
        let Some(expect_e) = self.get() else {
            return false;
        };
        if !matches!(expect_e, 'e' | 'E') {
            return false;
        }
        self.push_char(); // push e or E

        // (+ | -)?
        if let Some(expect_sign) = self.get() {
            if matches!(expect_sign, '+' | '-') {
                self.push_char(); // push + or -
            }
        }

        // (DecDigit | _)* DecDigit (DecDigit | _)*
        self.eat_dec_digit()
    }

    fn eat_literal_suffix(&mut self) {
        self.eat_identifier_or_keyword();
    }

    // String

    // CharacterLiteral ::= ' (~[' \ \n \r \t] | QuoteEscape | AsciiEscape | UnicodeEscape) ' Suffix?
    // QuoteEscape      ::= \' | \"
    // AsciiEscape      ::= \x OctDigit HexDigit | \n | \r | \t | \\ | \0
    // UnicodeEscape    ::= \u{ (HexDigit _*)1..6 }
    fn eat_charachter_literal(&mut self) -> bool {
        // '
        let Some(expect_first_quote) = self.get() else {
            return false;
        };
        if expect_first_quote != '\'' {
            return false;
        }
        self.push_char(); // push '

        // エスケープ処理した文字か判断
        if self.is_escape_start() {
            let Some(escape_prefix) = self.get_next() else {
                return false;
            };

            let result = match escape_prefix {
                '\'' | '\"' => self.eat_quote_escape(), // QuoteEscape ( \' | \" )
                'n' | 'r' | 't' | '\\' | '0' | 'x' => self.eat_ascii_escape(), // ( \x OctDigit HexDigit | \n | \r | \t | \\ | \0 )
                'u' => self.eat_unicode_escape(), // UnicodeEscape ::= \u{ (HexDigit _*)1..6 }
                _ => false,
            };
            if !result {
                return false;
            }
        } else {
            let Some(except_escape_char) = self.get() else {
                return false;
            };

            // ~[' \ \n \r \t] \のみは別途判断(この時点ではエスケープしているのか判断つかないので)
            if matches!(except_escape_char, '\'' | '\n' | '\r' | '\t') {
                return false;
            }

            self.push_char(); // push any char
        }

        // '
        let Some(last_c) = self.get() else {
            return false;
        };
        if last_c != '\'' {
            return false;
        }
        self.push_char(); // push '

        true
    }

    // StringLiteral  ::= " (~[" \ IsolatedCR] | QuoteEscape | AsciiEscape | UnicodeEscape | StringContinue)* " Suffix?
    // StringContinue ::= \ followed by \n

    // RawStringLiteral ::= r RawStringContent Suffix?
    // RawStringContent ::= " (~ IsolatedCR)* (non-greedy) " | # RawStringContent #

    // ByteLiteral  ::= b' (AsciiForChar | ByteEscape) ' Suffix?
    // AsciiForChar ::= any ASCII (i.e. 0x00 to 0x7F), except ', \, \n, \r or \t
    // ByteEscape   ::= \x HexDigit HexDigit | \n | \r | \t | \\ | \0 | \' | \"

    // ByteStringLiteral ::= b" (AsciiForString | ByteEscape | StringContinue)* " Suffix?
    // AsciiForString    ::= any ASCII (i.e 0x00 to 0x7F), except ", \ and IsolatedCR

    // RawByteStringLiteral ::= br RawByteStringContent Suffix?
    // RawByteStringContent ::= " AsciiForRaw* (non-greedy) " | # RawByteStringContent #
    // AsciiForRaw          ::= any ASCII (i.e. 0x00 to 0x7F) except IsolatedCR

    // CStringLiteral ::= c" (~[" \ IsolatedCR NUL] | ByteEscape except \0 or \x00 | UnicodeEscap except \u{0}, \u{00}, …, \u{000000} | StringContinue)* " Suffix?

    // RawCStringLiteral ::= cr RawCStringContent Suffix?
    // RawCStringContent ::= " ( ~ IsolatedCR NUL )* (non-greedy) " | # RawCStringContent #

    fn is_escape_start(&self) -> bool {
        let Some(c) = self.get() else {
            return false;
        };

        matches!(c, '\\')
    }

    fn eat_escape(&mut self) -> bool {
        if !self.is_escape_start() {
            return false;
        }

        self.push_char();
        true
    }

    // QuoteEscape ::= \' | \"
    fn eat_quote_escape(&mut self) -> bool {
        if !self.eat_escape() {
            return false;
        }

        let Some(c) = self.get() else {
            return false;
        };

        if !matches!(c, '\'' | '\"') {
            return false;
        }

        self.push_char();
        true
    }

    // AsciiEscape ::=  \x OctDigit HexDigit | \n | \r | \t | \\ | \0
    fn eat_ascii_escape(&mut self) -> bool {
        if !self.eat_escape() {
            return false;
        }

        let Some(c) = self.get() else {
            return false;
        };

        match c {
            'n' | 'r' | 't' | '\\' | '0' => self.push_char(),
            'x' => {
                self.push_char(); // push x

                // OctDigit
                if !self.eat_oct_digit() {
                    return false;
                }
                // HexDigit
                if !self.eat_hex_digit() {
                    return false;
                }
            }
            _ => return false,
        };

        true
    }

    // UnicodeEscape ::= \u{ (HexDigit _*)1..6 }
    fn eat_unicode_escape(&mut self) -> bool {
        // \
        if !self.eat_escape() {
            return false;
        }

        // u
        let Some(expect_u) = self.get() else {
            return false;
        };
        if expect_u == 'u' {
            return false;
        }
        self.push_char(); // push u

        // {
        let Some(expect_left_parentheses) = self.get() else {
            return false;
        };
        if expect_left_parentheses != '{' {
            return false;
        }
        self.push_char(); // push {

        // (HexDigit _*)1..6
        let mut hex_digit_count = 0;
        loop {
            // HexDigit
            if self.is_hex_digit() {
                self.push_char();
                hex_digit_count += 1;
            } else {
                break;
            }

            // _*
            while self.is_same('_') {
                self.push_char();
            }
        }
        if !(1..=6).contains(&hex_digit_count) {
            return false;
        }

        // }
        let Some(expect_right_parentheses) = self.get() else {
            return false;
        };
        if expect_right_parentheses != '{' {
            return false;
        }
        self.push_char(); // push }

        true
    }

    // ByteEscape ::= \x HexDigit HexDigit | \n | \r | \t | \\ | \0 | \' | \"
    fn eat_byte_escape(&mut self) -> bool {
        if !self.eat_escape() {
            return false;
        }

        let Some(c) = self.get() else {
            return false;
        };
        match c {
            'n' | 'r' | 't' | '\\' | '0' | '\'' | '\"' => {
                self.push_char();
                true
            }
            'x' => {
                self.push_char(); // push x

                // HexDigit
                if !self.eat_hex_digit() {
                    return false;
                }
                // HexDigit
                if !self.eat_hex_digit() {
                    return false;
                }
                true
            }
            _ => false,
        }
    }

    // Comment

    fn line_comment(&mut self) -> TokenKind {
        TokenKind::Comment
    }

    fn block_comment(&mut self) -> TokenKind {
        TokenKind::Comment
    }

    // other

    fn get(&self) -> Option<char> {
        Some(*self.code.get(self.position)?)
    }

    fn get_next(&self) -> Option<char> {
        Some(*self.code.get(self.position + 1)?)
    }

    fn push_char(&mut self) {
        let Some(c) = self.get() else {
            return;
        };
        self.token_buffer.push(c);
        self.next();
    }

    fn next(&mut self) {
        self.position += 1;
    }

    fn new_token(&mut self, token_kind: TokenKind) -> Token {
        let token = Token::new(token_kind, &self.token_buffer);
        self.token_buffer.clear();

        token
    }

    fn is_same(&self, check_c: char) -> bool {
        let Some(c) = self.get() else {
            return false;
        };

        check_c == c
    }
}
