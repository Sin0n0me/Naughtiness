mod ast_parse;
mod cst_parse;
mod expression;
mod lexer;
mod parser;

#[cfg(test)]
mod tests;

use cst_parse::cst_parser::CSTParser;
use nagi_errors::Error;
use nagi_lexer::lexer::Lexer;
use nagi_syntax_tree::cst::CSTNode;

pub fn parse(sorce_code: &str) -> Result<CSTNode, Error> {
    let mut lexer = Lexer::new(sorce_code);
    let mut parser = CSTParser::new(&lexer.tokenize());

    let parse_result = parser.parse();

    match parse_result {
        Ok(cst) => Ok(cst),
        Err(error) => Err(error),
    }
}
