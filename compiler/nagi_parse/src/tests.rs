#[cfg(test)]
mod test {
    use crate::cst_parse::cst_parser::CSTParser;
    use nagi_lexer::lexer::Lexer;

    fn parse(code: &str) {
        let mut lexer = Lexer::new(code);
        let mut parser = CSTParser::new(&lexer.tokenize());

        if let Ok(cst) = parser.parse() {
            cst.write_cst("test.json");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn check_() {
        parse("100 + fuga * 30 - 40000 / hogehoge + 200 - 100 * 10;");
    }
}
