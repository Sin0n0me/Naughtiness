use crate::token::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTNode {
    pub node_kind: ASTNodeKind,
}

impl ASTNode {
    pub fn new(node_kind: ASTNodeKind) -> Self {
        Self { node_kind }
    }

    pub fn write_ast(&self, file_name: &str) {
        let Ok(mut file) = File::create(file_name) else {
            return;
        };
        let Ok(data) = serde_json::to_string(self) else {
            return;
        };

        file.write_all(data.as_bytes()).unwrap();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ASTNodeKind {
    Crate {
        inner_attribute: Vec<ASTNode>,
        item: Vec<ASTNode>,
    },

    Factor {
        token: Token,
    },

    Literal {
        literal: Literal,
    },

    BinaryOperator {
        operator: BinaryOperator,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },

    InnerAttribute {
        attribute: Box<ASTNode>,
    },

    OuterAttribute {
        attribute: Box<ASTNode>,
    },

    Visibility {},

    Function {
        function_qualifiers: Box<ASTNode>,
        identifier: String,
        generic_params: Option<Box<ASTNode>>,
        function_parameters: Option<Box<ASTNode>>,
        function_return_type: Option<Box<ASTNode>>,
        where_clause: Option<Box<ASTNode>>,
        block_expression: Option<Box<ASTNode>>, // When None is ;
    },

    FunctionQualifiers {
        const_keyword: bool,
        async_keyword: bool,
        item_safety: Option<Box<ASTNode>>,
        extern_keyword: bool,
        abi: Option<Box<ASTNode>>,
    },

    Expression {
        expression: Box<ASTNode>,
    },

    ExpressionWithoutBlock {
        outer_attribute: Vec<ASTNode>,
        expression: Box<ASTNode>,
    },

    ExpressionWithBlock {
        outer_attribute: Vec<ASTNode>,
        expression_with_block: Box<ASTNode>,
    },

    PathExpression {
        expression: Box<ASTNode>,
    },

    // PathInExpression ::= `::`? PathExprSegment (`::` PathExprSegment)*
    PathInExpression {
        path_expr_segment: Box<ASTNode>,
        repeat_path_expr_segment: Vec<ASTNode>,
    },

    // PathExprSegment ::= PathIdentSegment (`::` GenericArgs)?
    PathExprSegment {
        path_ident_segment: Box<ASTNode>,
        generic_args: Option<Box<ASTNode>>,
    },

    // ReturnExpression ::= return (Expression)?
    ReturnExpression {
        expression: Option<Box<ASTNode>>,
    },

    // IfExpression ::= `if` Expression BlockExpression (`else` ( BlockExpression | IfExpression | IfLetExpression ) )?
    IfExpression {
        expression: Box<ASTNode>,
        block_expression: Box<ASTNode>,
        else_expression: Option<Box<ASTNode>>,
    },

    // IfLetExpression ::= `if` `let` Pattern `=` Scrutinee BlockExpression (`else` ( BlockExpression | IfExpression | IfLetExpression ) )?
    IfLetExpression {
        pattern: Box<ASTNode>,
        scrutinee: Box<ASTNode>,
        block_expression: Box<ASTNode>,
        else_expression: Option<Box<ASTNode>>,
    },

    BlockExpression {
        inner_attribute: Vec<ASTNode>,
        statements: Option<Box<ASTNode>>,
    },

    //
    IdentifierPattern {
        ref_keyword: bool,
        mut_keyword: bool,
        identifier: String,
        pattern_no_top_alt: Option<Box<ASTNode>>,
    },

    WildcardPattern {
        wildcard: Box<ASTNode>,
    },

    RestPattern {
        rest: Box<ASTNode>,
    },

    Statements {
        statements: Vec<ASTNode>,
    },

    Statement {
        statement: Option<Box<ASTNode>>,
    },

    LetStatement {
        outer_attribute: Vec<ASTNode>,
        rarity: Rarity,
        pattern_no_top_alt: Box<ASTNode>,
        type_expression: Option<Box<ASTNode>>,
        expression: Option<Box<ASTNode>>,
        block_expression: Option<Box<ASTNode>>,
    },
}
