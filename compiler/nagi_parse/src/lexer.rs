use nagi_cst::keywords::Keyword;
use nagi_cst::token::*;

#[derive(Debug)]
pub struct Lexer {
    token_list: Vec<Token>,
    position: usize,
    token_sorce_postion: Vec<(usize, usize)>,
}

impl Lexer {
    pub fn new(tokenized_token_list: &Vec<nagi_lexer::Token>) -> Self {
        let mut token_list = Vec::<Token>::new();
        let mut token_sorce_postion = Vec::<(usize, usize)>::new();
        let mut row = 1;
        let mut column = 1;
        for tokenized_token in tokenized_token_list.iter() {
            if let Some(token) = convert_token(tokenized_token) {
                token_list.push(token);
                token_sorce_postion.push((row, column));
            };

            for c in tokenized_token.token.chars() {
                column += 1;
                if c == '\n' {
                    row += 1;
                    column = 1;
                }
            }
        }

        Self {
            token_list,
            position: 0,
            token_sorce_postion,
        }
    }

    pub fn peek(&self) -> Token {
        self.token_list
            .get(self.position)
            .cloned()
            .unwrap_or(Token::Eof)
    }

    pub fn next(&mut self) -> Token {
        let token = self.peek();
        self.position += 1;

        token
    }

    pub fn peek_ahead(&self, ahead: usize) -> Token {
        self.token_list
            .get(self.position + ahead)
            .cloned()
            .unwrap_or(Token::Eof)
    }

    pub fn set_postion(&mut self, position: usize) {
        self.position = position;
    }

    pub fn next_glue(&mut self) -> Token {
        let Some((op, index)) = self.glue() else {
            return self.peek();
        };
        self.position += index;

        op
    }

    pub fn peek_glue(&mut self) -> Token {
        let Some((op, _)) = self.glue() else {
            return self.peek();
        };

        op
    }

    pub fn glue(&self) -> Option<(Token, usize)> {
        let first = self.peek();
        let res = match first {
            Token::Plus => match self.peek_ahead(1) {
                Token::Equal => Token::PlusEqual,
                _ => first,
            },
            Token::Minus => match self.peek_ahead(1) {
                Token::Equal => Token::MinusEqual,
                _ => first,
            },
            Token::Star => match self.peek_ahead(1) {
                Token::Equal => Token::StarEqual,
                _ => first,
            },
            Token::Slash => match self.peek_ahead(1) {
                Token::Equal => Token::SlashEqual,
                _ => first,
            },
            Token::Percent => match self.peek_ahead(1) {
                Token::Equal => Token::PercentEqual,
                _ => first,
            },
            Token::Caret => match self.peek_ahead(1) {
                Token::Equal => Token::CaretEqual,
                _ => first,
            },

            Token::Not => match self.peek_ahead(1) {
                Token::Equal => Token::NotEqual,
                _ => first,
            },
            Token::And => match self.peek_ahead(1) {
                Token::And => Token::AndAnd,
                Token::Equal => Token::AndEqual,
                _ => first,
            },
            Token::Or => match self.peek_ahead(1) {
                Token::Or => Token::Or,
                Token::Equal => Token::OrEqual,
                _ => first,
            },
            Token::GreaterThan => match self.peek_ahead(1) {
                Token::GreaterThan => match self.peek_ahead(2) {
                    Token::Equal => Token::RightShiftEqual,
                    _ => Token::RightShift,
                },
                Token::Equal => Token::GreaterThanOrEqual,
                _ => first,
            },
            Token::LessThan => match self.peek_ahead(1) {
                Token::LessThan => match self.peek_ahead(2) {
                    Token::Equal => Token::LeftShiftEqual,
                    _ => Token::LeftShift,
                },
                Token::Equal => Token::LessThanOrEqual,
                _ => first,
            },

            Token::Dot => match self.peek_ahead(1) {
                Token::Dot => match self.peek_ahead(2) {
                    Token::Dot => Token::DotDotDot,
                    Token::Equal => Token::DotDotEqual,
                    _ => Token::DotDot,
                },
                _ => first,
            },

            _ => return None,
        };

        if matches!(&res, first) {
            Some((res, 1))
        } else if matches!(
            &res,
            Token::LeftShiftEqual | Token::RightShiftEqual | Token::DotDotDot | Token::DotDotEqual
        ) {
            Some((res, 3))
        } else {
            Some((res, 2))
        }
    }

    pub fn is_operator(&self) -> bool {
        matches!(
            self.peek(),
            Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Percent
                | Token::Caret
                | Token::Not
                | Token::And
                | Token::Or
                | Token::Equal
        )
    }

    pub fn get_token_position(&self) -> usize {
        self.position
    }

    pub fn get_sorce_position(&self) -> (usize, usize) {
        self.token_sorce_postion
            .get(self.position)
            .cloned()
            .unwrap_or((0, 0))
    }
}

fn convert_token(token: &nagi_lexer::Token) -> Option<Token> {
    let res = match &token.token_kind {
        nagi_lexer::TokenKind::Identifier(identifier) => {
            if let Some(keyword) = Keyword::from_str(identifier) {
                Token::Keyword(keyword)
            } else {
                Token::Identifier(identifier.to_string())
            }
        }

        nagi_lexer::TokenKind::Literal(literal_kind) => match literal_kind {
            nagi_lexer::LiteralKind::BinLiteral
            | nagi_lexer::LiteralKind::OctLiteral
            | nagi_lexer::LiteralKind::DecLiteral
            | nagi_lexer::LiteralKind::HexLiteral => {
                Token::Literal(Literal::new(LiteralKind::Integer, &token.token))
            }
            nagi_lexer::LiteralKind::FloatLiteral(_) => {
                Token::Literal(Literal::new(LiteralKind::Float, &token.token))
            }
            _ => return None,
        },

        nagi_lexer::TokenKind::LeftParenthesis => {
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        }
        nagi_lexer::TokenKind::LeftBrackets => Token::LeftParenthesis(LeftParenthesis::Brackets),
        nagi_lexer::TokenKind::LeftBrace => Token::LeftParenthesis(LeftParenthesis::Brace),
        nagi_lexer::TokenKind::RightParenthesis => {
            Token::RightParenthesis(RightParenthesis::Parenthesis)
        }
        nagi_lexer::TokenKind::RightBrackets => Token::RightParenthesis(RightParenthesis::Brackets),
        nagi_lexer::TokenKind::RightBrace => Token::RightParenthesis(RightParenthesis::Brace),

        nagi_lexer::TokenKind::Equal => Token::Equal,
        nagi_lexer::TokenKind::Plus => Token::Plus,
        nagi_lexer::TokenKind::Minus => Token::Minus,
        nagi_lexer::TokenKind::Star => Token::Star,
        nagi_lexer::TokenKind::Slash => Token::Slash,
        nagi_lexer::TokenKind::Percent => Token::Percent,
        nagi_lexer::TokenKind::Not => Token::Not,
        nagi_lexer::TokenKind::And => Token::And,
        nagi_lexer::TokenKind::Or => Token::Or,
        nagi_lexer::TokenKind::GreaterThan => Token::GreaterThan,
        nagi_lexer::TokenKind::LessThan => Token::LessThan,

        nagi_lexer::TokenKind::At => Token::At,
        nagi_lexer::TokenKind::Dot => Token::Dot,
        nagi_lexer::TokenKind::Pound => Token::Pound,
        nagi_lexer::TokenKind::Comma => Token::Colon,
        nagi_lexer::TokenKind::Semicolon => Token::Semicolon,
        nagi_lexer::TokenKind::Dollar => Token::Dollar,
        nagi_lexer::TokenKind::Question => Token::Question,
        nagi_lexer::TokenKind::Tilde => Token::Tilde,
        nagi_lexer::TokenKind::Underscore => Token::Underscore,

        nagi_lexer::TokenKind::Eof => Token::Eof,

        nagi_lexer::TokenKind::WhiteSpace => return None,
        _ => panic!("{:?}", token),
    };

    Some(res)
}
