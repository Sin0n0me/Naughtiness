use crate::lexer::Lexer;
use crate::parser::*;

use nagi_syntax_tree::ast::*;
use nagi_syntax_tree::keywords::Keyword;
use nagi_syntax_tree::token::*;

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

pub struct ASTParser {
    lexer: Lexer,
    memo: HashMap<ParseMemoKey, Option<ASTNode>>,
    min_bp: u16,
    last_write_memo: ParseMemoKey,
}

// TODO 機能ごとの分割
impl ASTParser {
    pub fn new(token_list: &Vec<nagi_lexer::Token>) -> Self {
        Self {
            lexer: Lexer::new(token_list),
            memo: HashMap::new(),
            min_bp: 0,
            last_write_memo: ParseMemoKey {
                position: 0,
                rule: "".to_string(),
            },
        }
    }
}
