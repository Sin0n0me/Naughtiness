use nagi_cst::token::*;

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
        | Token::GreaterThanOrEqual => (7, 6),

        Token::AndAnd => (6, 5),
        Token::OrOr => (5, 4),
        Token::DotDot | Token::DotDotEqual => (4, 3),
        Token::LeftAllow => (3, 2),

        // Assign
        Token::Equal => (2, 1),
        Token::PlusEqual => (2, 1),
        Token::MinusEqual => (2, 1),
        Token::StarEqual => (2, 1),
        Token::SlashEqual => (2, 1),
        Token::PercentEqual => (2, 1),
        Token::CaretEqual => (2, 1),
        Token::AndEqual => (2, 1),
        Token::OrEqual => (2, 1),

        _ => return None,
    };

    Some(res)
}

pub fn postfix_binding_power(op: &Token) -> Option<(u16, ())> {
    let res = match op {
        _ => return None,
    };

    Some(res)
}
