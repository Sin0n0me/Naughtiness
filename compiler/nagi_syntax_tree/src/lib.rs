pub mod ast;
pub mod cst;
pub mod hst;
pub mod keywords;
pub mod token;

pub enum SyntaxTree {
    AST(ast::ASTNode),
    CST(cst::CSTNode),
    HST(hst::HSTNode),
}
