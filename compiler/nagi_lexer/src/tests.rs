#[cfg(test)]
mod test {
    use crate::lexer::Lexer;
    use crate::{LiteralKind, Token, TokenKind};

    // 正常パターン
    fn check_equal(test_code: &str, token_kind: TokenKind) {
        let token = Lexer::new(test_code).next_token();
        println!("{}", test_code);
        assert_eq!(token_kind, token.unwrap().token_kind);
    }

    // 異常パターン
    fn check_not_equal(test_code: &str, token_kind: TokenKind) {
        let token = Lexer::new(test_code).next_token();
        println!("{}", test_code);
        assert_ne!(token_kind, token.unwrap().token_kind);
    }

    // トークンが
    fn check_token_order(test_code: &str, token_kind_list: Vec<TokenKind>) {
        let token_list = Lexer::new(test_code).tokenize();

        assert_eq!(token_list.len(), token_kind_list.len());
        for (count, tk1) in token_kind_list.iter().enumerate() {
            let tk2 = &token_list.get(count).unwrap().token_kind;
            assert_eq!(tk1, tk2);
        }
    }

    #[test]
    fn identifier_or_keyword() {
        check_equal("hogeFuga", TokenKind::Identifier("".to_string()));
        check_equal("_hoge", TokenKind::Identifier("".to_string()));
        check_equal("hoge123", TokenKind::Identifier("".to_string()));
        check_equal("__hoge__", TokenKind::Identifier("".to_string()));
        check_equal("_____", TokenKind::Identifier("".to_string()));
        check_equal("こんにちは", TokenKind::Identifier("".to_string()));

        check_not_equal("0123456789", TokenKind::Identifier("".to_string()));
        check_not_equal("_", TokenKind::Identifier("".to_string()));
    }

    #[test]
    fn literal_bin() {
        check_equal("0b01011", TokenKind::Literal(LiteralKind::BinLiteral));
        check_equal("0b_1_0_", TokenKind::Literal(LiteralKind::BinLiteral));

        check_not_equal("0b", TokenKind::Literal(LiteralKind::BinLiteral));
        check_not_equal("0b01210", TokenKind::Literal(LiteralKind::BinLiteral));
        check_not_equal("010101", TokenKind::Literal(LiteralKind::BinLiteral));
        check_not_equal("0B01010", TokenKind::Literal(LiteralKind::BinLiteral));
        check_not_equal("0b____", TokenKind::Literal(LiteralKind::BinLiteral));
    }

    #[test]
    fn literal_oct() {
        check_equal("0o01234567", TokenKind::Literal(LiteralKind::OctLiteral));
        check_equal("0o_0_7_", TokenKind::Literal(LiteralKind::OctLiteral));

        check_not_equal("0o", TokenKind::Literal(LiteralKind::OctLiteral));
        check_not_equal("0o012345678", TokenKind::Literal(LiteralKind::OctLiteral));
        check_not_equal("01234567", TokenKind::Literal(LiteralKind::OctLiteral));
        check_not_equal("0O01234567", TokenKind::Literal(LiteralKind::OctLiteral));
        check_not_equal("0o____", TokenKind::Literal(LiteralKind::OctLiteral));
    }

    #[test]
    fn literal_dec() {
        check_equal("1234567890", TokenKind::Literal(LiteralKind::DecLiteral));
        check_equal("0_9_", TokenKind::Literal(LiteralKind::DecLiteral));
        check_equal("0", TokenKind::Literal(LiteralKind::DecLiteral));
        check_equal("100u64", TokenKind::Literal(LiteralKind::DecLiteral));

        check_equal("02468ACE", TokenKind::Literal(LiteralKind::DecLiteral)); // この時点では 10進数 + 有効なsuffix かは判定しない

        check_not_equal("_246", TokenKind::Literal(LiteralKind::DecLiteral));
    }
    #[test]
    fn literal_hex() {
        check_equal("0x123abcDEF", TokenKind::Literal(LiteralKind::HexLiteral));
        check_equal("0xFf00", TokenKind::Literal(LiteralKind::HexLiteral));
        check_equal("0x_0_F_", TokenKind::Literal(LiteralKind::HexLiteral));

        check_equal("0xEF_GH", TokenKind::Literal(LiteralKind::HexLiteral)); // この時点では 16進数 + 有効なsuffix かは判定しない

        check_not_equal("0x", TokenKind::Literal(LiteralKind::HexLiteral));
        check_not_equal("123ABC", TokenKind::Literal(LiteralKind::HexLiteral));
        check_not_equal("0XABC", TokenKind::Literal(LiteralKind::HexLiteral));
        check_not_equal("0x____", TokenKind::Literal(LiteralKind::HexLiteral));
    }

    #[test]
    fn literal_float() {
        check_equal(
            "123.456",
            TokenKind::Literal(LiteralKind::FloatLiteral(false)),
        );
        check_equal("0.1", TokenKind::Literal(LiteralKind::FloatLiteral(false)));
        check_equal("0.", TokenKind::Literal(LiteralKind::FloatLiteral(false)));
        check_equal(
            "1.23e45",
            TokenKind::Literal(LiteralKind::FloatLiteral(true)),
        );
        check_equal(
            "12E+34",
            TokenKind::Literal(LiteralKind::FloatLiteral(true)),
        );
        check_equal(
            "12E+34_f64",
            TokenKind::Literal(LiteralKind::FloatLiteral(true)),
        );

        check_not_equal(
            "1.23e45",
            TokenKind::Literal(LiteralKind::FloatLiteral(false)),
        );
        check_not_equal("0", TokenKind::Literal(LiteralKind::FloatLiteral(false)));
    }
}
