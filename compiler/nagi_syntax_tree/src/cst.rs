use crate::token::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CSTNode {
    pub node_kind: CSTNodeKind,
    pub children: Vec<CSTNode>,
}

impl CSTNode {
    pub fn new(node_kind: CSTNodeKind, children: Vec<CSTNode>) -> Self {
        Self {
            node_kind,
            children,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CSTNodeKind {
    None, // 一時

    Crate {
        inner_attributes: Vec<CSTNode>,
        items: Vec<CSTNode>,
    },

    Factor {
        token: Token,
        index: usize,
    },

    Operator {
        token: Token,
        left: Option<Box<CSTNode>>,
        right: Option<Box<CSTNode>>,
        index: usize,
    },

    Literal {
        literal: Literal,
        index: usize,
    },

    //
    // Attribute
    //

    // InnerAttribute ::= `#` `!` `[` Attribute `]`
    InnerAttribute {
        attribute: Box<CSTNode>,
    },

    // OuterAttribute ::= `#` `[` Attribute `]`
    OuterAttribute {
        attribute: Box<CSTNode>,
    },

    // Attribute ::= SimplePath AttributeInput?  | `unsafe` `(` SimplePath AttributeInput? `)`
    Attribute,

    //
    // Item
    //

    // Visibility ::= `pub`
    //              | `pub` `(` `crate` `)`
    //              | `pub` `(` `self` `)`
    //              | `pub` `(` `super` `)`
    //              | `pub` `(` `in` SimplePath `)`
    Visibility {
        pub_keyword: Box<CSTNode>,
    },

    VisItem {
        visibility: Option<Box<CSTNode>>,
        item: Box<CSTNode>,
    },

    // Item ::= OuterAttribute* VisItem | MacroItem
    Item,

    //
    // Function
    //

    // Function ::= FunctionQualifiers `fn` Identifier GenericParams?
    //             `(` FunctionParameters? `)`
    //             FunctionReturnType? WhereClause?
    //             ( BlockExpression | `;` )
    Function {
        function_qualifiers: Box<CSTNode>,
        identifier: Box<CSTNode>,
        generic_params: Option<Box<CSTNode>>,
        function_parameters: Option<Box<CSTNode>>,
        function_return_type: Option<Box<CSTNode>>,
        where_clause: Option<Box<CSTNode>>,
        block_expression_or_semicolon: Box<CSTNode>,
    },

    // FunctionQualifiers ::= `const`? `async`? ItemSafety? (`extern` Abi?)?
    FunctionQualifiers {
        const_keyword: Option<Box<CSTNode>>,
        async_keyword: Option<Box<CSTNode>>,
        item_safety: Option<Box<CSTNode>>,
        extern_keyword: Option<Box<CSTNode>>,
        abi: Option<Box<CSTNode>>,
    },

    // FunctionParameters ::= SelfParam `,`?
    FunctionParametersSelfOnly {
        self_param: Box<CSTNode>,
    },

    // FunctionParameters ::= (SelfParam `,`)? FunctionParam (`,` FunctionParam)* `,`?
    FunctionParameters {
        self_param: Option<Box<CSTNode>>,
        function_param: Vec<CSTNode>,
    },

    // FunctionParam ::= OuterAttribute* ( FunctionParamPattern | `...` | Type )
    FunctionParam {
        outer_attribute: Vec<CSTNode>,
        pattern: Box<CSTNode>,
    },

    FunctionParamPattern {
        pattern_no_top_alt: Box<CSTNode>,
        pattern: Box<CSTNode>,
    },

    //
    SelfParam {
        outer_attribute: Vec<CSTNode>,
        self_kind: Box<CSTNode>,
    },

    TypedSelf {
        mut_keyword: Option<Box<CSTNode>>,
        type_expr: Box<CSTNode>,
    },

    //
    // Type
    //

    // Type
    Type {
        type_pattern: Box<CSTNode>,
    },

    TypeNoBounds {
        type_pattern: Box<CSTNode>,
    },

    // ParenthesizedType ::= `(` Type `)`
    ParenthesizedType {
        type_expression: Box<CSTNode>,
    },

    TypePath {
        type_path_segment: Vec<CSTNode>,
    },

    TypePathFnInputs {
        type_expr: Vec<CSTNode>,
    },

    //
    // Generic
    //

    // GenericArgsConst ::= BlockExpression | LiteralExpression | `-` LiteralExpression | SimplePathSegment
    GenericArgsConst {
        expression: Box<CSTNode>,
    },

    // QualifiedPathInExpression ::= QualifiedPathType (`::` PathExprSegment)+
    QualifiedPathInExpression {
        qualified_path_type: Box<CSTNode>,
        path_expr_segment: Vec<CSTNode>,
    },

    //
    // Expression
    //

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
        outer_attribute: Vec<CSTNode>,
        expression: Box<CSTNode>,
    },

    // ExpressionWithoutBlock ::= OuterAttribute*
    //                           (
    //                                LiteralExpression | PathExpression | OperatorExpression | GroupedExpression | ArrayExpression
    //                              | AwaitExpression | IndexExpression | TupleExpression | TupleIndexingExpression | StructExpression
    //                              | CallExpression | MethodCallExpression | FieldExpression | ClosureExpression | AsyncBlockExpression
    //                              | ContinueExpression | BreakExpression | RangeExpression | ReturnExpression | UnderscoreExpression | MacroInvocation
    //                           )
    ExpressionWithBlock {
        outer_attribute: Vec<CSTNode>,
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

    // PathInExpression ::= `::`? PathExprSegment (`::` PathExprSegment)*
    PathInExpression {
        path_expr_segment: Vec<CSTNode>,
    },

    // PathExprSegment ::= PathIdentSegment (`::` GenericArgs)?
    PathExprSegment {
        path_ident_segment: Box<CSTNode>,
        generic_args: Option<Box<CSTNode>>,
    },

    PathIdentSegment {
        path_ident_segment: Box<CSTNode>,
    },

    // GroupedExpression ::= `(` Expression `)`
    GroupedExpression {
        expression: Box<CSTNode>,
    },

    // StructExpression ::= StructExprStruct | StructExprTuple | StructExprUnit
    StructExpression {
        expression: Box<CSTNode>,
    },

    // StructExprStruct ::= PathInExpression `{` (StructExprFields | StructBase)? `}`
    StructExprStruct {
        path_in_expression: Box<CSTNode>,
        expression: Option<Box<CSTNode>>,
    },

    // StructExprFields ::= StructExprField (, StructExprField)* (, StructBase | ,?)
    StructExprFields {
        struct_expr_filed: Vec<CSTNode>,
        struct_base: Option<Box<CSTNode>>,
    },

    // StructExprField  ::= OuterAttribute* ( Identifier | (Identifier |TUPLE_INDEX) `:` Expression )
    StructExprField1 {
        outer_attribute: Vec<CSTNode>,
        identifier: Box<CSTNode>,
    },

    StructExprField2 {
        outer_attribute: Vec<CSTNode>,
        identifier_or_tuple: Box<CSTNode>,
        expression: Box<CSTNode>,
    },

    // StructBase ::= `..` Expression
    StructBase {
        expression: Box<CSTNode>,
    },

    //  CallExpression ::= Expression `(` CallParams? `)`
    CallExpression {
        expression: Box<CSTNode>,
        call_params: Option<Box<CSTNode>>,
    },

    // CallParams ::= Expression ( `,` Expression )* `,`?
    CallParams {
        expression: Vec<CSTNode>,
    },

    // ReturnExpression ::= return (Expression)?
    ReturnExpression {
        expression: Option<Box<CSTNode>>,
    },

    // IfExpression ::= `if` Expression BlockExpression (`else` ( BlockExpression | IfExpression | IfLetExpression ) )?
    IfExpression {
        expression: Box<CSTNode>,
        block_expression: Box<CSTNode>,
        else_expression: Option<Box<CSTNode>>,
    },

    // IfLetExpression ::= `if` `let` Pattern `=` Scrutinee BlockExpression (`else` ( BlockExpression | IfExpression | IfLetExpression ) )?
    IfLetExpression {
        pattern: Box<CSTNode>,
        scrutinee: Box<CSTNode>,
        block_expression: Box<CSTNode>,
        else_expression: Option<Box<CSTNode>>,
    },

    // Statements ::= Statement+ | Statement+ ExpressionWithoutBlock | ExpressionWithoutBlock
    Statements {
        statements: Vec<CSTNode>,
    },

    // Statement ::= `;` | Item | LetStatement | ExpressionStatement | MacroInvocationSemi
    Statement {
        statement: Box<CSTNode>,
    },

    BlockExpression {
        inner_attribute: Vec<CSTNode>,
        statements: Option<Box<CSTNode>>,
    },

    // LetStatement ::= OuterAttribute* (`ur` | `sr` | `nr` | `let`)
    //                  PatternNoTopAlt ( `:` Type )?
    //                  (`=` Expression ( `else` BlockExpression)? )? `;`
    LetStatement {
        outer_attribute: Vec<CSTNode>,
        rarity: Box<CSTNode>,
        pattern_no_top_alt: Box<CSTNode>,
        type_expression: Option<Box<CSTNode>>,
        expression: Option<Box<CSTNode>>,
        block_expression: Option<Box<CSTNode>>,
    },

    ExpressionStatement {
        expression: Box<CSTNode>,
    },

    // Pattern

    // Pattern ::= `|`? PatternNoTopAlt ( `|` PatternNoTopAlt )*
    Pattern {
        pattern: Vec<CSTNode>,
    },

    // PatternNoTopAlt ::= PatternWithoutRange | RangePattern
    PatternNoTopAlt {
        pattern: Box<CSTNode>,
    },

    PatternWithoutRange {
        pattern: Box<CSTNode>,
    },

    LiteralPattern {
        literal: Literal,
    },

    //
    IdentifierPattern {
        ref_keyword: Option<Box<CSTNode>>,
        mut_keyword: Option<Box<CSTNode>>,
        identifier: Box<CSTNode>,
        pattern_no_top_alt: Option<Box<CSTNode>>,
    },

    WildcardPattern {
        wildcard: Box<CSTNode>,
    },

    RestPattern {
        rest: Box<CSTNode>,
    },

    // LoopExpression ::= LoopLabel?
    //                  (
    //                    InfiniteLoopExpression
    //                  | PredicateLoopExpression
    //                  | PredicatePatternLoopExpression
    //                  | IteratorLoopExpression
    //                  | LabelBlockExpression
    //                  )
    LoopExpression {
        loop_label: Option<Box<CSTNode>>,
        loop_expression: Box<CSTNode>,
    },

    // InfiniteLoopExpression ::= `loop` BlockExpression
    InfiniteLoopExpression {
        block_expression: Box<CSTNode>,
    },

    // PredicateLoopExpression ::= `while` Expression BlockExpression
    PredicateLoopExpression {
        expression: Box<CSTNode>,
        block_expression: Box<CSTNode>,
    },
}
