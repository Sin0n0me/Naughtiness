use crate::expression::Expression;
use crate::token::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSTNode {
    pub node_kind: CSTNodeKind,
    pub children: Vec<CSTNode>,
}

impl CSTNode {
    pub fn new(node_kind: CSTNodeKind) -> Self {
        Self {
            node_kind,
            children: Vec::new(),
        }
    }

    pub fn write_cst(&self, file_name: &str) {
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
pub enum CSTNodeKind {
    Factor {
        token: Token,
        row: usize,
        column: usize,
    },

    Literal {
        literal: Literal,
        row: usize,
        column: usize,
    },

    Attribute,

    // Expression ::= ExpressionWithoutBlock | ExpressionWithBlock
    Expression {
        expression: Box<CSTNode>,
    },

    // ExpressionWithoutBlock ::= OuterAttribute*
    //                            (
    //                              LiteralExpression | PathExpression | OperatorExpression | GroupedExpression
    //                            | ArrayExpression | AwaitExpression | IndexExpression | TupleExpression
    //                            | TupleIndexingExpression | StructExpression | CallExpression | MethodCallExpression
    //                            | FieldExpression | ClosureExpression | AsyncBlockExpression | ContinueExpression
    //                            | BreakExpression | RangeExpression | ReturnExpression | UnderscoreExpression | MacroInvocation
    //                            )
    ExpressionWithoutBlock {
        outer_attribute: Option<Vec<CSTNode>>,
        expression_without_block: Box<CSTNode>,
    },

    // ExpressionWithoutBlock ::= OuterAttribute*
    //                           (
    //                                LiteralExpression | PathExpression | OperatorExpression | GroupedExpression | ArrayExpression
    //                              | AwaitExpression | IndexExpression | TupleExpression | TupleIndexingExpression | StructExpression
    //                              | CallExpression | MethodCallExpression | FieldExpression | ClosureExpression | AsyncBlockExpression
    //                              | ContinueExpression | BreakExpression | RangeExpression | ReturnExpression | UnderscoreExpression | MacroInvocation
    //                           )
    ExpressionWithBlock {
        outer_attribute: Option<Vec<CSTNode>>,
        expression_with_block: Box<CSTNode>,
    },

    // LiteralExpression ::=  CharacterLiteral
    //                      | StringLiteral
    //                      | RawStringLiteral
    //                      | ByteLiteral
    //                      | ByteStringLiteral
    //                      | RawByteStringLiteral
    //                      | CStringLiteral
    //                      | RawCStringLiteral
    //                      | IntgerLiteral
    //                      | FloatLiteral
    //                      | true
    //                      | false
    LiteralExpression {
        literal: Box<CSTNode>,
    },

    // PathExpression ::= PathInExpression | QualifiedPathInExpression
    PathExpression {
        path_in_expression: Box<CSTNode>,
    },

    GroupedExpression {
        left_parenthesis: Box<CSTNode>,
        expression: Box<CSTNode>,
        right_parenthesis: Box<CSTNode>,
    },

    //  CallExpression ::= Expression `(` CallParams? `)`
    CallExpression {
        expression: Box<CSTNode>,
        left_parenthesis: Box<CSTNode>,
        call_params: Option<Box<CSTNode>>,
        right_parenthesis: Box<CSTNode>,
    },

    // CallParams     ::= Expression ( `,` Expression )* `,`?
    CallParams {
        expression: Box<CSTNode>,
        comma_and_expression: Vec<(CSTNode, CSTNode)>,
        comma: Option<Box<CSTNode>>,
    },

    // ReturnExpression ::= return (Expression)?
    ReturnExpression {
        return_keyword: Box<CSTNode>,
        expression: Option<Box<CSTNode>>,
    },

    // Statements ::= Statement+ | Statement+ ExpressionWithoutBlock | ExpressionWithoutBlock
    Statements,

    // Statement ::= `;` | Item | LetStatement | ExpressionStatement | MacroInvocationSemi
    Statement {
        statement: Box<CSTNode>,
    },

    //
    Item,

    BlockExpression {
        left_brace: Box<CSTNode>,
        inner_attribute: Vec<CSTNode>,
        statements: Option<Box<CSTNode>>,
        right_brace: Box<CSTNode>,
    },

    // LetStatement ::= OuterAttribute* (`ur` | `sr` | `nr` | `let`)
    //                  PatternNoTopAlt ( `:` Type )?
    //                  (`=` Expression ( `else` BlockExpression)? )? `;`
    LetStatement {
        outer_attribute: Vec<CSTNode>,
        rarity: Box<CSTNode>,
        pattern_no_top_alt: Box<CSTNode>,
        colon: Option<Box<CSTNode>>,
        type_expression: Option<Box<CSTNode>>,
        equal: Option<Box<CSTNode>>,
        expression: Option<Box<CSTNode>>,
        else_keyword: Option<Box<CSTNode>>,
        block_expression: Option<Box<CSTNode>>,
        semicolon: Box<CSTNode>,
    },

    // Pattern
    LiteralPattern {
        literal: Literal,
    },

    //
    IdentifierPattern {
        ref_keyword: Option<Box<CSTNode>>,
        mut_keyword: Option<Box<CSTNode>>,
        identifier: Box<CSTNode>,
        at_symbol: Option<Box<CSTNode>>,
        pattern_no_top_alt: Option<Box<CSTNode>>,
    },
}
