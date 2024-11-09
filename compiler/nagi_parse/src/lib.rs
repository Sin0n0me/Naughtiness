mod ast_parse;
mod cst_parse;
mod expression;
mod lexer;
mod parser;

#[cfg(test)]
mod tests;

use cst_parse::cst_parser::CSTParser;
use nagi_lexer::lexer::Lexer;

pub fn parse(sorce_code: &str) {
    let mut lexer = Lexer::new(sorce_code);
    let mut parser = CSTParser::new(&lexer.tokenize());

    if let Ok(cst) = parser.parse() {
        cst.write_cst("test.json");
    } else {
        println!("error!");
    }
}
