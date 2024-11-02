use crate::cst::CSTNode;
use serde::{Deserialize, Serialize};

// Expression ::= ExpressionWithoutBlock | ExpressionWithBlock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
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
}
