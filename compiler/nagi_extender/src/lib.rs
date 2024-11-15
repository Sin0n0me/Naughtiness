use nagi_syntax_tree::ast::ASTNode;
use nagi_syntax_tree::cst::CSTNode;
use nagi_syntax_tree::hst::HSTNode;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Write};

// CSTとASTをまぜまぜする

pub struct Extender {
    e: i32,
}

impl Extender {}

pub fn import_ast(file_path: &str) -> Result<ASTNode, ()> {
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let ast: ASTNode = serde_json::from_reader(reader).unwrap();

    Ok(ast)
}

pub fn export_ast(file_name: &str, ast: &ASTNode) -> Result<(), ()> {
    let Ok(mut file) = File::create(file_name) else {
        return Err(());
    };
    let Ok(data) = serde_json::to_string(ast) else {
        return Err(());
    };

    file.write_all(data.as_bytes()).unwrap();

    Ok(())
}

//pub fn add_syntax_tree(cst: ASTNode, ast: ASTNode) -> HSTNode {}

//pub fn merge_syntax_tree(cst: ASTNode, ast: ASTNode, condition: i32) -> HSTNode {}

//pub fn replace_syntax_tree() -> HSTNode {}
