use nagi_syntax_tree::token::*;

pub fn prefix_binding_power(op: &Token) -> Option<((), u16)> {
    let res = match op {
        Token::Not => ((), 15),
        Token::Minus => ((), 15),
        _ => return None,
    };

    Some(res)
}

pub fn infix_binding_power(op: &Token) -> Option<(u16, u16)> {
    let res = match op {
        Token::Dot => (15, 16),

        Token::Star | Token::Slash | Token::Percent => (13, 14),
        Token::Plus | Token::Minus => (12, 13),

        Token::LeftShift | Token::RightShift => (11, 10),

        Token::And => (10, 9),
        Token::Caret => (9, 8),
        Token::Or => (8, 7),

        // == != < > <= >=
        Token::EqualEqual
        | Token::NotEqual
        | Token::LessThan
        | Token::GreaterThan
        | Token::LessThanOrEqual
        | Token::GreaterThanOrEqual => (6, 7),

        Token::AndAnd => (5, 6),
        Token::OrOr => (4, 5),
        Token::DotDot | Token::DotDotEqual => (3, 4),
        Token::LeftAllow => (2, 3),

        // Assignment
        Token::Equal => (1, 2),
        Token::PlusEqual => (1, 2),
        Token::MinusEqual => (1, 2),
        Token::StarEqual => (1, 2),
        Token::SlashEqual => (1, 2),
        Token::PercentEqual => (1, 2),
        Token::CaretEqual => (1, 2),
        Token::AndEqual => (1, 2),
        Token::OrEqual => (1, 2),

        _ => return None,
    };

    Some(res)
}

pub fn postfix_binding_power(op: &Token) -> Option<(u16, ())> {
    None // TODO
}

pub fn is_operator(token: &Token) -> bool {
    prefix_binding_power(token).is_some()
        || infix_binding_power(token).is_some()
        || postfix_binding_power(token).is_some()
}
