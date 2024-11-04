#[cfg(test)]
mod test {
    use crate::cst_parse::cst_parser::CSTParser;
    use nagi_lexer::lexer::Lexer;

    fn parse(code: &str) {
        let mut lexer = Lexer::new(code);
        let mut parser = CSTParser::new(&lexer.tokenize());

        match parser.parse() {
            Ok(cst) => cst.write_cst("test.json"),
            Err(error) => {
                parser.error(error);
                assert!(false);
            }
        }
    }

    #[test]
    fn check_() {
        parse("{ let a = 100 + 300 * 30 - 40000 / 1000 + 200 - 100 * 10; }");
    }
}
