mod ast_parse;
mod cst_parse;
mod lexer;

#[cfg(test)]
mod tests;

use cst_parse::cst_parser::CSTParser;
use nagi_lexer::lexer::Lexer;

pub fn parse() {
    let mut lexer = Lexer::new("1+1/10-9*10");
    let mut parser = CSTParser::new(&lexer.tokenize());

    if let Ok(cst) = parser.parse() {
        cst.write_cst("test.json");
    } else {
        println!("error!");
    }
}
