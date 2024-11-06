use crate::lexer::Lexer;

use nagi_syntax_tree::ast::*;
use nagi_syntax_tree::keywords::Keyword;
use nagi_syntax_tree::token::*;

use std::collections::HashMap;

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct ParseMemoKey {
    position: usize,
    rule: String,
}

enum MemoResult<T> {
    None,
    Recursive,
    Some(T),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    E,
    Recursive,
    NotExpected, // 期待したトークンではなかった
}

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

    pub fn parse(&mut self) -> Result<ASTNode, Error> {
        let mut node = self.expression()?;
        Ok(node)
    }

    //
    // Attributes
    //

    // InnerAttribute ::= `#` `!` `[` Attribute `]`
    fn inner_attribute(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("InnerAttribute");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // `#`
        let pound = Box::new(self.make_factor());
        if matches!(self.lexer.next(), Token::Pound) {
            return Err(Error::NotExpected);
        }

        // `!`
        let exclamation = Box::new(self.make_factor());
        if matches!(self.lexer.next(), Token::Not) {
            return Err(Error::NotExpected);
        }

        // `[`
        let left_brackets = Box::new(self.make_factor());
        if matches!(
            self.lexer.next(),
            Token::LeftParenthesis(LeftParenthesis::Brackets)
        ) {
            return Err(Error::NotExpected);
        }

        let attribute = Box::new(self.attribute()?);

        // `]`
        let right_brackets = Box::new(self.make_factor());
        if matches!(
            self.lexer.next(),
            Token::RightParenthesis(RightParenthesis::Brackets)
        ) {
            return Err(Error::NotExpected);
        }

        let node = ASTNode::new(ASTNodeKind::InnerAttribute {
            pound,
            exclamation,
            left_brackets,
            attribute,
            right_brackets,
        });
        self.write_memo(&key, Some(&node));

        Ok(node)
    }

    // OuterAttribute ::= `#` `[` Attribute `]`
    fn outer_attribute(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("OuterAttribute");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // `#`
        if !matches!(self.lexer.peek(), Token::Pound) {
            return Err(Error::NotExpected);
        }
        let pound = Box::new(self.make_factor());
        self.lexer.next();

        // `[`
        let left_brackets = Box::new(self.make_factor());
        if !matches!(
            self.lexer.next(),
            Token::LeftParenthesis(LeftParenthesis::Brackets)
        ) {
            return Err(Error::NotExpected);
        }

        let attribute = Box::new(self.attribute()?);

        // `]`
        let right_brackets = Box::new(self.make_factor());
        if !matches!(
            self.lexer.next(),
            Token::RightParenthesis(RightParenthesis::Brackets)
        ) {
            return Err(Error::NotExpected);
        }

        let node = ASTNode::new(ASTNodeKind::OuterAttribute {
            pound,
            left_brackets,
            attribute,
            right_brackets,
        });
        self.write_memo(&key, Some(&node));

        Ok(node)
    }

    // Attribute ::= SimplePath AttributeInput?  | `unsafe` `(` SimplePath AttributeInput? `)`
    fn attribute(&mut self) -> Result<ASTNode, Error> {
        Err(Error::E)
    }

    // AttributeInput ::= DelimTokenTree | `=` Expression
    fn attribute_input(&mut self) -> Result<ASTNode, Error> {
        Err(Error::E)
    }

    // Visibility ::= `pub`
    //              | `pub` `(` `crate` `)`
    //              | `pub` `(` `self` `)`
    //              | `pub` `(` `super` `)`
    //              | `pub` `(` `in` SimplePath `)`
    fn visibility(&mut self) -> Result<ASTNode, Error> {
        // `pub`
        let mut pub_keyword = ASTNode::new(ASTNodeKind::Visibility {
            pub_keyword: Box::new(self.make_factor()),
        });
        if matches!(self.lexer.next(), Token::Keyword(Keyword::Pub)) {
            return Err(Error::NotExpected);
        }

        if matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return Ok(pub_keyword);
        }

        Ok(pub_keyword)
    }

    // Item ::= OuterAttribute* VisItem | MacroItem
    fn item(&mut self) -> Result<ASTNode, Error> {
        self.outer_attribute();

        self.vis_item()
    }

    // VisItem ::= Visibility?
    //           (
    //             Module
    fn vis_item(&mut self) -> Result<ASTNode, Error> {
        self.visibility();

        self.function()?;

        Err(Error::E)
    }

    // Functions

    // Function ::= FunctionQualifiers `fn` Identifier GenericParams?
    //             `(` FunctionParameters? `)`
    //             FunctionReturnType? WhereClause?
    //             ( BlockExpression | `;` )
    fn function(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("Function");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // FunctionQualifiers
        let function_qualifiers = self.function_qualifiers()?;

        // `fn`
        let fn_keyword = Box::new(self.make_factor());
        if !matches!(self.lexer.next(), Token::Keyword(Keyword::Fn)) {
            return Err(Error::E);
        }

        // Identifier
        if !matches!(self.lexer.next(), Token::Identifier(_)) {
            return Err(Error::E);
        }

        // GenericParams?
        if let Ok(expr) = self.generic_params() {
            // TODO
        }

        // `(`
        let left_parenthesis = Box::new(self.make_factor());
        if !matches!(
            self.lexer.next(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return Err(Error::NotExpected);
        }

        // FunctionParameters?
        let mut call_params = None;
        if let Ok(res) = self.call_params() {
            call_params = Some(Box::new(res));
        }

        // `)`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return Err(Error::NotExpected);
        }

        // FunctionReturnType?
        if let Ok(expr) = self.function_return_type() {
            // TODO
        }

        // WhereClause?
        // TODO

        // ( BlockExpression | `;` )
        if let Ok(expr) = self.block_expression() {}

        if let Token::Semicolon = self.lexer.peek() {}

        Err(Error::E)
    }

    // FunctionQualifiers ::= `const`? `async`? ItemSafety? (`extern` Abi?)?
    fn function_qualifiers(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("FunctionQualifiers");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        let mut const_keyword = None;
        let mut async_keyword = None;
        let mut item_safety = None;
        let mut extern_keyword = None;
        let mut abi = None;

        // `const`?
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Const)) {
            const_keyword = Some(Box::new(self.make_factor()));
            self.lexer.next();
        }

        // `async`?
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Async)) {
            async_keyword = Some(Box::new(self.make_factor()));
            self.lexer.next();
        }

        // ItemSafety?
        if let Ok(expr) = self.item_safety() {
            item_safety = Some(Box::new(expr));
            self.lexer.next();
        }

        // (`extern` `Abi`?)?
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Extern)) {
            extern_keyword = Some(Box::new(self.make_factor()));
            self.lexer.next();

            if let Ok(expr) = self.abi() {
                abi = Some(Box::new(expr));
                self.lexer.next();
            }
        }

        Err(Error::E)
    }

    // ItemSafety ::= `safe` | `unsafe`
    fn item_safety(&mut self) -> Result<ASTNode, Error> {
        match self.lexer.peek() {
            Token::Keyword(Keyword::Unsafe) => {
                self.lexer.next();
                Ok(self.make_factor())
            }
            Token::Identifier(identifier) => {
                if identifier != "safe" {
                    return Err(Error::NotExpected);
                }
                self.lexer.next();

                Ok(self.make_factor())
            }
            _ => Err(Error::NotExpected),
        }
    }

    // Abi ::= STRING_LITERAL | RAW_STRING_LITERAL
    fn abi(&mut self) -> Result<ASTNode, Error> {
        match self.lexer.peek() {
            Token::Literal(literal) => {
                if !matches!(literal.literal_kind, LiteralKind::Str | LiteralKind::StrRaw) {
                    return Err(Error::NotExpected);
                }

                self.lexer.next();
                Ok(self.make_factor())
            }
            _ => Err(Error::NotExpected),
        }
    }

    // GenericParams
    fn generic_params(&mut self) -> Result<ASTNode, Error> {
        Err(Error::E)
    }

    // FunctionParameters ::= SelfParam `,`? | (SelfParam `,`)? FunctionParam (`,` FunctionParam)* `,`?
    fn function_parameters(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("FunctionParameters");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // SelfParam
        self.self_param();

        //
        if matches!(self.lexer.peek(), Token::Comma) {
            // TODO
        }

        // FunctionParam
        self.function_param()?;

        // (`,` FunctionParam)*
        while let Token::Comma = self.lexer.peek() {
            self.function_param();
        }

        Err(Error::E)
    }

    // SelfParam ::= OuterAttribute* ( ShorthandSelf | TypedSelf )
    fn self_param(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("Selfparam");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // OuterAttribute*
        self.outer_attribute();

        // ShorthandSelf
        if let Ok(expr) = self.shorthand_self() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // TypedSelf

        self.backtrack(&key);

        Err(Error::E)
    }

    // ShorthandSelf ::= (`&` | `&` Lifetime)? `mut`? `self`
    fn shorthand_self(&mut self) -> Result<ASTNode, Error> {
        // (`&` | `&` Lifetime)?
        if matches!(self.lexer.peek(), Token::And) {
            // TODO
        }

        // `mut`?
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Mut)) {
            // TODO
        }

        // `self`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::SelfValue)) {
            return Err(Error::E);
        }

        Err(Error::E)
    }

    // TypedSelf ::= `mut`? `self` `:` Type
    fn typed_self(&mut self) -> Result<ASTNode, Error> {
        let mut node = self.make_factor();

        // `mut`?
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Mut)) {
            // TODO
        }

        // `self`
        if !matches!(self.lexer.next(), Token::Keyword(Keyword::SelfValue)) {
            return Err(Error::NotExpected);
        }

        // `:`
        if !matches!(self.lexer.next(), Token::Colon) {
            return Err(Error::NotExpected);
        }

        // Type
        // TODO

        Ok(node)
    }

    // FunctionParam ::= OuterAttribute* ( FunctionParamPattern | `...` | Type )
    fn function_param(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("FunctionParam");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // OuterAttribute*
        self.outer_attribute();

        // `...`
        if let Token::DotDotDot = self.lexer.peek() {
            self.lexer.next();
            // TODO
        }

        // FunctionParamPattern
        if let Ok(expr) = self.function_param_pattern() {
            return Ok(expr);
        }

        // Type

        Err(Error::E)
    }

    // FunctionParamPattern ::= PatternNoTopAlt `:` ( Type | `...` )
    fn function_param_pattern(&mut self) -> Result<ASTNode, Error> {
        // PatternNoTopAlt
        self.pattern_no_top_alt()?;

        // `:`
        if !matches!(self.lexer.next(), Token::Colon) {
            return Err(Error::E);
        }

        // ( Type | `...` )
        match self.lexer.peek_glue() {
            Token::DotDotDot => (),
            _ => (),
        };

        // TODO
        Err(Error::E)
    }

    // FunctionReturnType ::= `->` Type
    fn function_return_type(&mut self) -> Result<ASTNode, Error> {
        Err(Error::E)
    }

    // Structs

    //
    // Expressions
    //

    // Expression ::= ExpressionWithoutBlock | ExpressionWithBlock
    fn expression(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("Expression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        if let Ok(expr) = self.expression_without_block() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        if let Ok(expr) = self.expression_with_block() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        Err(Error::E)
    }

    // ExpressionWithoutBlock ::= OuterAttribute*
    //                           (
    //                                LiteralExpression | PathExpression | OperatorExpression | GroupedExpression | ArrayExpression
    //                              | AwaitExpression | IndexExpression | TupleExpression | TupleIndexingExpression | StructExpression
    //                              | CallExpression | MethodCallExpression | FieldExpression | ClosureExpression | AsyncBlockExpression
    //                              | ContinueExpression | BreakExpression | RangeExpression | ReturnExpression | UnderscoreExpression | MacroInvocation
    //                           )
    fn expression_without_block(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("ExpressionWithoutBlock");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        if let Ok(expr) = self.outer_attribute() {
            // TODO
        } else {
            self.backtrack(&key);
        }

        // OperatorExpression
        if let Ok(expr) = self.operator_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // LiteralExpression
        if let Ok(expr) = self.literal_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // PathExpression
        if let Ok(expr) = self.path_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // GroupedExpression
        if let Ok(expr) = self.grouped_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // StructExpression
        if let Ok(expr) = self.struct_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // CallExpression
        if let Ok(expr) = self.call_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // MethodCallExpression
        if let Ok(expr) = self.method_call_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // ReturnExpression
        if let Ok(expr) = self.return_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        Err(Error::E)
    }

    fn literal_expression(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("LiteralExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        let pos = self.lexer.get_sorce_position();
        let result = match self.lexer.next() {
            Token::Literal(literal) => Ok(ASTNode::new(ASTNodeKind::Literal {
                literal,
                row: pos.0,
                column: pos.1,
            })),

            Token::Keyword(Keyword::True) => Ok(ASTNode::new(ASTNodeKind::Literal {
                literal: Literal::new(LiteralKind::Bool(true), ""),
                row: pos.0,
                column: pos.1,
            })),
            Token::Keyword(Keyword::False) => Ok(ASTNode::new(ASTNodeKind::Literal {
                literal: Literal::new(LiteralKind::Bool(false), ""),
                row: pos.0,
                column: pos.1,
            })),

            _ => Err(Error::NotExpected),
        };

        if let Ok(expr) = &result {
            self.write_memo(&key, Some(&expr));
        }

        result
    }

    // PathExpression ::= PathInExpression | QualifiedPathInExpression
    fn path_expression(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("PathExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // PathInExpression
        if let Ok(expr) = self.path_in_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // QualifiedPathInExpression
        if let Ok(expr) = self.qualified_path_in_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        Err(Error::E)
    }

    // PathInExpression ::= `::`? PathExprSegment (`::` PathExprSegment)*
    fn path_in_expression(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("PathInExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        let mut path_separater = None;

        //  `::`?
        let glue_token = self.lexer.peek_glue();
        if matches!(glue_token, Token::PathSeparater) {
            let pos = self.lexer.get_sorce_position();
            path_separater = Some(Box::new(ASTNode::new(ASTNodeKind::Factor {
                token: glue_token,
                row: pos.0,
                column: pos.1,
            })));
            self.lexer.next_glue();
        }

        // PathExprSegment
        let path_expr_segment = Box::new(self.path_expr_segment()?);

        // (`::` PathExprSegment)*
        let mut repeat_path_expr_segment = Vec::<(ASTNode, ASTNode)>::new();
        loop {
            // `::`
            let pos = self.lexer.get_sorce_position();
            if !matches!(self.lexer.peek_glue(), Token::PathSeparater) {
                break;
            }
            self.lexer.next_glue();

            // PathExprSegment
            let Ok(expr) = self.path_expr_segment() else {
                break;
            };

            repeat_path_expr_segment.push((
                ASTNode::new(ASTNodeKind::Factor {
                    token: Token::PathSeparater,
                    row: pos.0,
                    column: pos.1,
                }),
                expr,
            ));
        }

        let node = ASTNode::new(ASTNodeKind::PathInExpression {
            path_separater,
            path_expr_segment,
            repeat_path_expr_segment,
        });
        self.write_memo(&key, Some(&node));

        Ok(node)
    }

    // PathExprSegment ::= PathIdentSegment (`::` GenericArgs)?
    fn path_expr_segment(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("PathExprSegment");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // PathIdentSegment
        let path_ident_segment = Box::new(self.path_ident_segment()?);

        //  (`::` GenericArgs)?
        let mut generic_args = None;
        if matches!(self.lexer.peek_glue(), Token::PathSeparater) {

            // GenericArgs
        }

        Ok(ASTNode::new(ASTNodeKind::PathExprSegment {
            path_ident_segment,
            generic_args,
        }))
    }

    // PathIdentSegment   ::= Identifier | `super` | `self` | `Self` | `crate` | `$crate`
    fn path_ident_segment(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("PathIdentSegment");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        let node = self.make_factor();
        match self.lexer.peek() {
            Token::Identifier(_) => {
                self.lexer.next();
                self.write_memo(&key, Some(&node));
                Ok(node)
            }
            Token::Keyword(keyword) => match keyword {
                Keyword::Super | Keyword::SelfValue | Keyword::SelfType | Keyword::Crate => {
                    self.lexer.next();
                    self.write_memo(&key, Some(&node));
                    Ok(node)
                }
                _ => Err(Error::E),
            },

            _ => Err(Error::E),
        }
    }

    //
    fn qualified_path_in_expression(&mut self) -> Result<ASTNode, Error> {
        Err(Error::E)
    }

    // Pratt parsing
    // OperatorExpression
    fn operator_expression(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("OperatorExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        let min_bp = self.min_bp;

        // 前置演算子
        let mut lhs: ASTNode = match self.lexer.peek() {
            Token::LeftParenthesis(LeftParenthesis::Parenthesis) => {
                self.min_bp = 0;
                let mut node = self.make_factor();
                self.lexer.next();

                node.children.push(self.operator_expression()?);

                node
            }

            _ if self.lexer.is_operator() => {
                let op = self.lexer.peek_glue();
                let Some(((), right_bp)) = prefix_binding_power(&op) else {
                    assert!(false);
                    return Err(Error::NotExpected);
                };

                self.min_bp = right_bp; // 次の再帰のために保存
                let mut node = self.make_factor();
                self.lexer.next_glue();

                node.children.push(self.operator_expression()?);

                node
            }
            _ => {
                // Expressionの再帰用に呼び出し元だけを削除
                self.memo.remove(&self.make_key("Expression"));
                self.memo.remove(&self.make_key("ExpressionWithoutBlock"));
                self.expression()?
            }
        };

        loop {
            let op_pos = self.lexer.get_sorce_position();
            let op = self.lexer.peek_glue();

            match &op {
                Token::Eof => break,
                _ if !op.is_operator() => return Err(Error::NotExpected),
                _ => (),
            };

            // 後置演算子
            if let Some((left_bp, ())) = postfix_binding_power(&op) {
                if left_bp < min_bp {
                    break;
                }
                self.lexer.next_glue();

                let mut node = ASTNode::new(ASTNodeKind::Factor {
                    token: op,
                    row: op_pos.0,
                    column: op_pos.1,
                });
                node.children.push(lhs);

                lhs = node;
                continue;
            }

            // 中置演算子
            if let Some((left_bp, right_bp)) = infix_binding_power(&op) {
                if left_bp < min_bp {
                    break;
                }
                self.lexer.next_glue();

                self.min_bp = right_bp; // 次の再帰のために保存
                let rhs = self.operator_expression()?;
                let mut node = ASTNode::new(ASTNodeKind::Factor {
                    token: op,
                    row: op_pos.0,
                    column: op_pos.1,
                });

                node.children.push(lhs);
                node.children.push(rhs);

                lhs = node;
                continue;
            }

            break;
        }

        self.write_memo(&key, Some(&lhs));
        Ok(lhs)
    }

    // BorrowExpression ::= (`&`|`&&`) Expression
    //                    | (`&`|`&&`) `mut` Expression
    //                    | (`&`|`&&`) `raw` `const` Expression
    //                    | (`&`|`&&`) `raw` `mut` Expression
    fn borrow_expression(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("BrrowExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // (`&`|`&&`)
        if matches!(self.lexer.next_glue(), Token::And | Token::AndAnd) {
            return Err(Error::E);
        }

        if let Token::Keyword(keyword) = self.lexer.peek() {
            if !matches!(keyword, Keyword::Mut) {
                return Err(Error::E);
            }
            self.lexer.next();
        } else if let Token::Identifier(identifier) = self.lexer.peek() {
            let token = self.lexer.peek();
            if let Token::Keyword(keyword) = token {}
        }

        self.expression()
    }

    fn dereference_expression(&mut self) -> Result<ASTNode, Error> {
        self.expression()
    }

    // GroupedExpression ::= `(` Expression `)`
    fn grouped_expression(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("GroupedExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // `(`
        let left_parenthesis = Box::new(self.make_factor());
        if !matches!(
            self.lexer.next(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return Err(Error::NotExpected);
        }

        // Expression
        let expression = Box::new(self.expression()?);

        // `)`
        let right_parenthesis = Box::new(self.make_factor());
        if !matches!(
            self.lexer.next(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return Err(Error::NotExpected);
        }

        Ok(ASTNode::new(ASTNodeKind::GroupedExpression {
            left_parenthesis,
            expression,
            right_parenthesis,
        }))
    }

    // StructExpression ::= StructExprStruct | StructExprTuple | StructExprUnit
    fn struct_expression(&mut self) -> Result<ASTNode, Error> {
        Err(Error::E)
    }

    // StructExprStruct ::= PathInExpression `{` (StructExprFields | StructBase)? `}`

    // StructExprFields ::= StructExprField (, StructExprField)* (, StructBase | ,?)

    // StructExprField  ::= OuterAttribute* ( IDENTIFIER | (IDENTIFIER |TUPLE_INDEX) `:` Expression )

    // StructBase       ::= `..` Expression

    // StructExprTuple  ::=  PathInExpression `(` ( Expression (, Expression)* ,? )? `)`

    // StructExprUnit   ::= PathInExpression

    // CallExpression ::= Expression `(` CallParams? `)`
    fn call_expression(&mut self) -> Result<ASTNode, Error> {
        // Expression
        let expression = Box::new(self.expression()?);

        // `(`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return Err(Error::NotExpected);
        }
        let left_parenthesis = Box::new(self.make_factor());
        self.lexer.next();

        // CallParams?
        let mut call_params = None;
        if let Ok(res) = self.call_params() {
            call_params = Some(Box::new(res));
        }

        // `)`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return Err(Error::NotExpected);
        }
        let right_parenthesis = Box::new(self.make_factor());
        self.lexer.next();

        Ok(ASTNode::new(ASTNodeKind::CallExpression {
            expression,
            left_parenthesis,
            call_params,
            right_parenthesis,
        }))
    }

    // CallParams     ::= Expression ( `,` Expression )* `,`?
    fn call_params(&mut self) -> Result<ASTNode, Error> {
        let expression = Box::new(self.expression()?);

        // ( `,` Expression )*
        let mut comma_and_expression = Vec::<(ASTNode, ASTNode)>::new();
        loop {
            if !matches!(self.lexer.peek(), Token::Comma) {
                break;
            }
            let comma = self.make_factor();
            self.lexer.next();

            let Ok(expr) = self.expression() else {
                break;
            };

            comma_and_expression.push((comma, expr));
        }

        let comma = if matches!(self.lexer.peek(), Token::Comma) {
            Some(Box::new(self.make_factor()))
        } else {
            None
        };

        Ok(ASTNode::new(ASTNodeKind::CallParams {
            expression,
            comma_and_expression,
            comma,
        }))
    }

    // MethodCallExpression ::= Expression `.` PathExprSegment `(` CallParams? `)`
    fn method_call_expression(&mut self) -> Result<ASTNode, Error> {
        self.expression()?;

        self.call_params();

        Err(Error::E)
    }

    // ReturnExpression ::= return Expression?
    fn return_expression(&mut self) -> Result<ASTNode, Error> {
        // `return`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Return)) {
            return Err(Error::E);
        }
        let return_keyword = Box::new(self.make_factor());
        self.lexer.next();

        // Expression?
        let mut expression = None;
        if let Ok(expr) = self.expression() {
            expression = Some(Box::new(expr));
        }

        Ok(ASTNode::new(ASTNodeKind::ReturnExpression {
            return_keyword,
            expression,
        }))
    }

    //IfExpression ::= `if` Expression BlockExpression (`else` ( BlockExpression | IfExpression | IfLetExpression ) )?
    fn if_expression(&mut self) -> Result<ASTNode, Error> {
        // `if`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::If)) {
            return Err(Error::E);
        }
        let if_keyword = Box::new(self.make_factor());
        self.lexer.next();

        // Expression
        let expression = self.expression()?;

        // BlockExpression
        let block_expression = self.block_expression()?;

        // `else`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Else)) {
            return Ok(ASTNode::new(ASTNodeKind::Item));
        }
        let else_keyword = self.make_factor();

        //  ( BlockExpression | IfExpression | IfLetExpression )
        self.block_expression();

        self.if_expression();

        self.if_let_expression();

        Err(Error::E)
    }

    // IfLetExpression ::= `if` `let` Pattern `=` Scrutinee BlockExpression
    //                   ( else ( BlockExpression | IfExpression | IfLetExpression ) )?
    fn if_let_expression(&mut self) -> Result<ASTNode, Error> {
        Err(Error::E)
    }

    // MatchExpression ::= `match` Scrutinee `{` InnerAttribute* MatchArms? `}`
    fn match_expression(&mut self) -> Result<ASTNode, Error> {
        // `match`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Match)) {
            return Err(Error::E);
        }
        self.make_factor();
        self.lexer.next();

        // Scrutinee

        // `{`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Brace)
        ) {
            return Err(Error::E);
        }
        self.make_factor();
        self.lexer.next();

        // InnerAttribute*

        // MatchArms?

        // `}`
        if !matches!(
            self.lexer.peek(),
            Token::RightParenthesis(RightParenthesis::Brace)
        ) {
            return Err(Error::E);
        }
        self.make_factor();
        self.lexer.next();

        Err(Error::E)
    }

    fn scrutinee(&mut self) -> Result<ASTNode, Error> {
        let Ok(expression) = self.expression() else {
            return Err(Error::E);
        };

        Err(Error::E)
    }

    // ExpressionWithBlock ::= OuterAttribute*
    //                        (
    //                          BlockExpression | ConstBlockExpression | UnsafeBlockExpression | LoopExpression
    //                        | IfExpression | IfLetExpression | MatchExpression
    //                        )
    fn expression_with_block(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("ExpressionWithBlock");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        self.outer_attribute();

        // BlockExpression
        if let Ok(expr) = self.block_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // IfExpression
        if let Ok(expr) = self.if_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // IfLetExpression
        if let Ok(expr) = self.if_let_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        Err(Error::E)
    }

    // BlockExpression ::=  `{` InnerAttribute* Statements? `}`
    fn block_expression(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("BlockExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // `{`
        let left_brace = Box::new(self.make_factor());
        if !matches!(
            self.lexer.next(),
            Token::LeftParenthesis(LeftParenthesis::Brace)
        ) {
            return Err(Error::NotExpected);
        }

        let mut statements = None;
        if let Ok(expr) = self.statements() {
            statements = Some(Box::new(expr));
        }

        // `}`
        let right_brace = Box::new(self.make_factor());
        if !matches!(
            self.lexer.next(),
            Token::RightParenthesis(RightParenthesis::Brace)
        ) {
            return Err(Error::NotExpected);
        }

        let expr = ASTNode::new(ASTNodeKind::BlockExpression {
            left_brace,
            inner_attribute: Vec::new(),
            statements,
            right_brace,
        });
        self.write_memo(&key, Some(&expr));

        Ok(expr)
    }

    // Statements ::= Statement+ | Statement+ ExpressionWithoutBlock | ExpressionWithoutBlock
    fn statements(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("Statements");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // Statement+ | Statement+ ExpressionWithoutBlock
        let mut node = ASTNode::new(ASTNodeKind::Statements);
        if let Ok(expr1) = self.statement() {
            node.children.push(expr1);

            while let Ok(expr2) = self.statement() {
                node.children.push(expr2);
            }

            // ExpressionWithoutBlock
            if let Ok(expr3) = self.expression_without_block() {
                node.children.push(expr3);
            }

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // ExpressionWithoutBlock
        if let Ok(expr) = self.expression_without_block() {
            node.children.push(expr);
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        self.backtrack(&key);
        Err(Error::E)
    }

    //
    // Statement
    //

    // Statement ::= `;` | Item | LetStatement | ExpressionStatement | MacroInvocationSemi
    fn statement(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("Statement");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // ;
        if matches!(self.lexer.next(), Token::Semicolon) {
            let expr = ASTNode::new(ASTNodeKind::Statement {
                statement: Box::new(self.make_factor()),
            });
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // Item

        // LetStatement
        if let Ok(expr) = self.let_statement() {
            self.write_memo(&key, Some(&expr));
            return Ok(ASTNode::new(ASTNodeKind::Statement {
                statement: Box::new(expr),
            }));
        }
        self.backtrack(&key);

        // ExpressionStatement
        if let Ok(expr) = self.expression_statement() {
            self.write_memo(&key, Some(&expr));
            return Ok(ASTNode::new(ASTNodeKind::Statement {
                statement: Box::new(expr),
            }));
        }
        self.backtrack(&key);

        // MacroInvocationSemi

        Err(Error::E)
    }

    // LetStatement ::= OuterAttribute* (`ur` | `sr` | `nr` | `let`)
    //                  PatternNoTopAlt ( `:` Type )?
    //                  (`=` Expression ( `else` BlockExpression)? )? `;`
    fn let_statement(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("LetStatement");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        let mut colon = None;
        let mut type_expression = None;
        let mut equal = None;
        let mut expression = None;
        let mut else_keyword = None;
        let mut block_expression = None;

        // OuterAttribute*
        self.outer_attribute();

        // (`ur` | `sr` | `nr` | `let`)
        let rarity = match self.lexer.peek() {
            Token::Keyword(keyword) => match keyword {
                Keyword::Ur | Keyword::Sr | Keyword::Nr | Keyword::Let => {
                    Box::new(self.make_factor())
                }
                _ => return Err(Error::NotExpected),
            },
            _ => return Err(Error::NotExpected),
        };
        self.lexer.next();

        // PatternNoTopAlt
        let pattern_no_top_alt = Box::new(self.pattern_no_top_alt()?);

        // ( `:` Type )?
        if matches!(self.lexer.peek(), Token::Colon) {
            colon = Some(Box::new(self.make_factor()));

            self.lexer.next();

            // Type
        }

        //  (`=` Expression ( `else` BlockExpression)? )? `;`
        if matches!(self.lexer.peek(), Token::Equal) {
            // `=`
            equal = Some(Box::new(self.make_factor()));
            self.lexer.next();

            // Expression
            expression = Some(Box::new(self.expression()?));

            // `else`
            if let Token::Keyword(keyword) = self.lexer.peek() {
                if !matches!(keyword, Keyword::Else) {
                    return Err(Error::E);
                }

                else_keyword = Some(Box::new(self.make_factor()));
                self.lexer.next();

                // BlockExpression
                block_expression = Some(Box::new(self.block_expression()?))
            }
        }

        // ;
        let semicolon = Box::new(self.make_factor());
        if !matches!(self.lexer.next(), Token::Semicolon) {
            return Err(Error::E);
        }

        let expr = ASTNode::new(ASTNodeKind::LetStatement {
            outer_attribute: Vec::new(),
            rarity,
            pattern_no_top_alt,
            colon,
            type_expression,
            equal,
            expression,
            else_keyword,
            block_expression,
            semicolon,
        });
        self.write_memo(&key, Some(&expr));

        Ok(expr)
    }

    // ExpressionStatement ::= ExpressionWithoutBlock `;` | ExpressionWithBlock `;`?
    fn expression_statement(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("ExpressionStatement");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // ExpressionWithoutBlock `;`
        if let Ok(mut expr) = self.expression_without_block() {
            if matches!(self.lexer.peek(), Token::Semicolon) {
                expr.children.push(self.make_factor());
                self.write_memo(&key, Some(&expr));
                return Ok(expr);
            }
        }
        self.backtrack(&key);

        // ExpressionWithBlock `;`?
        if let Ok(expr) = self.expression_with_block() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        Err(Error::E)
    }

    //
    // 以下Pattern
    //

    // Pattern ::= `|`? PatternNoTopAlt ( `|` PatternNoTopAlt )*
    fn pattern(&mut self) -> Result<ASTNode, Error> {
        let mut or_token = None;

        // `|`?
        if let Token::Or = self.lexer.peek() {
            or_token = Some(Box::new(self.make_factor()));
            self.lexer.next();
        }

        self.pattern_no_top_alt()?;

        while let Ok(expr_pattern_no_top_alt) = self.pattern_no_top_alt() {
            //
        }

        Err(Error::E)
    }

    // PatternNoTopAlt ::= PatternWithoutRange | RangePattern
    fn pattern_no_top_alt(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("PatternNoTopAlt");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        if let Ok(expr) = self.pattern_without_range() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        Err(Error::E)
    }

    // PatternWithoutRange ::= LiteralPattern | IdentifierPattern | WildcardPattern | RestPattern |
    //                         ReferencePattern | StructPattern | TupleStructPattern | TuplePattern | GroupedPattern |
    //                         SlicePattern | PathPattern | MacroInvocation
    fn pattern_without_range(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("PatternWithoutRange");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // LiteralPattern
        if let Ok(expr) = self.literal_pattern() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // IdentifierPattern
        if let Ok(expr) = self.identifier_pattern() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // WildcardPattern
        if let Ok(expr) = self.wildcard_pattern() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // RestPattern
        if let Ok(expr) = self.rest_pattern() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        // ReferencePattern
        if let Ok(expr) = self.reference_pattern() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }
        self.backtrack(&key);

        Err(Error::E)
    }

    // LiteralPattern ::= `true` | `false`
    //                  | CHAR_LITERAL
    //                  | BYTE_LITERAL
    //                  | STRING_LITERAL
    //                  | RAW_STRING_LITERAL
    //                  | BYTE_STRING_LITERAL
    //                  | RAW_BYTE_STRING_LITERAL
    //                  | C_STRING_LITERAL
    //                  | RAW_C_STRING_LITERAL
    //                  | `-`? INTEGER_LITERAL
    //                  | `-`? FLOAT_LITERAL
    fn literal_pattern(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("LiteralPattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        match self.lexer.next() {
            Token::Keyword(keyword) => match keyword {
                Keyword::True => {
                    let node = ASTNode::new(ASTNodeKind::LiteralPattern {
                        literal: Literal::new(LiteralKind::Bool(true), ""),
                    });
                    self.write_memo(&key, Some(&node));
                    Ok(node)
                }

                Keyword::False => {
                    let node = ASTNode::new(ASTNodeKind::LiteralPattern {
                        literal: Literal::new(LiteralKind::Bool(false), ""),
                    });
                    self.write_memo(&key, Some(&node));
                    Ok(node)
                }
                _ => Err(Error::NotExpected),
            },
            Token::Literal(literal) => {
                let node = ASTNode::new(ASTNodeKind::LiteralPattern { literal });
                self.write_memo(&key, Some(&node));
                Ok(node)
            }
            //
            Token::Minus => {
                let Token::Literal(literal) = self.lexer.next() else {
                    return Err(Error::E);
                };

                match literal.literal_kind {
                    LiteralKind::Integer | LiteralKind::Float => {
                        let node = ASTNode::new(ASTNodeKind::LiteralPattern { literal });
                        self.write_memo(&key, Some(&node));
                        Ok(node)
                    }
                    _ => Err(Error::E),
                }
            }

            _ => Err(Error::E),
        }
    }

    // IdentifierPattern ::= `ref`? `mut`? Identifier (`@` PatternNoTopAlt )?
    fn identifier_pattern(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("IdentifierPattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        let mut ref_keyword = None;
        let mut mut_keyword = None;
        let mut at_symbol = None;
        let mut pattern_no_top_alt = None;

        // `ref`?
        if let Token::Keyword(keyword) = self.lexer.peek() {
            if matches!(keyword, Keyword::Ref) {
                ref_keyword = Some(Box::new(self.make_factor()));
                self.lexer.next();
            }
        }

        // `mut`?
        if let Token::Keyword(keyword) = self.lexer.peek() {
            if matches!(keyword, Keyword::Mut) {
                mut_keyword = Some(Box::new(self.make_factor()));
                self.lexer.next();
            }
        }

        // Identifier
        let identifier = match self.lexer.peek() {
            Token::Identifier(_) => Box::new(self.make_factor()),
            _ => return Err(Error::E),
        };
        self.lexer.next();

        // (`@` PatternNoTopAlt )?
        if matches!(self.lexer.peek(), Token::At) {
            at_symbol = Some(Box::new(self.make_factor()));
            self.lexer.next();
            pattern_no_top_alt = Some(Box::new(self.pattern_no_top_alt()?));
        }

        let node = ASTNode::new(ASTNodeKind::IdentifierPattern {
            ref_keyword,
            mut_keyword,
            identifier,
            at_symbol,
            pattern_no_top_alt,
        });
        self.write_memo(&key, Some(&node));

        Ok(node)
    }

    // WildcardPattern ::= `_`
    fn wildcard_pattern(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("WildcardPattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        match self.lexer.peek() {
            Token::Underscore => {
                let node = self.make_factor();
                self.write_memo(&key, Some(&node));
                Ok(node)
            }
            _ => Err(Error::NotExpected),
        }
    }

    // RestPattern ::= `..`
    fn rest_pattern(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("RestPattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        match self.lexer.next_glue() {
            Token::DotDot => {
                let node = self.make_factor();
                self.write_memo(&key, Some(&node));
                Ok(node)
            }
            _ => Err(Error::NotExpected),
        }
    }

    // ReferencePattern ::= (`&`|`&&`) mut? PatternWithoutRange
    fn reference_pattern(&mut self) -> Result<ASTNode, Error> {
        let key = self.make_key("ReferencePattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // (`&`|`&&`)
        if !matches!(self.lexer.next_glue(), Token::And | Token::AndAnd) {
            return Err(Error::NotExpected);
        }

        // mut?
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Mut)) {
            return Err(Error::NotExpected);
        }

        // PatternWithoutRange
        self.pattern_without_range()
    }

    //
    //
    //

    fn make_factor(&self) -> ASTNode {
        let pos = self.lexer.get_sorce_position();
        let token = self.lexer.peek();
        ASTNode::new(ASTNodeKind::Factor {
            token,
            row: pos.0,
            column: pos.1,
        })
    }

    fn make_key(&self, rule: &str) -> ParseMemoKey {
        ParseMemoKey {
            position: self.lexer.get_token_position(),
            rule: rule.to_string(),
        }
    }

    fn write_memo(&mut self, key: &ParseMemoKey, node: Option<&ASTNode>) {
        let mut file = OpenOptions::new().append(true).open("log.txt").unwrap();
        if let Err(e) = writeln!(
            file,
            "{} -> {} -> {:?}",
            key.rule,
            key.position,
            self.lexer.peek()
        ) {
            eprintln!("Couldn't write to file: {}", e);
        }

        self.last_write_memo = key.clone();
        self.memo.insert(key.clone(), node.cloned());
    }

    fn get_memo(&self, key: &ParseMemoKey) -> MemoResult<ASTNode> {
        let Some(node) = self.memo.get(&key) else {
            return MemoResult::None;
        };

        let Some(value) = node.clone() else {
            return MemoResult::Recursive;
        };

        MemoResult::Some(value)
    }

    fn backtrack(&mut self, key: &ParseMemoKey) {
        let mut file = OpenOptions::new().append(true).open("log.txt").unwrap();
        if let Err(e) = writeln!(
            file,
            "{} <- {} <- {:?}",
            key.rule,
            key.position,
            self.lexer.peek()
        ) {
            eprintln!("Couldn't write to file: {}", e);
        }

        self.lexer.set_postion(key.position);
    }

    fn err(&mut self, error_type: Error) -> Result<(), Error> {
        // backtrack

        Err(error_type)
    }

    pub fn error(&self, error_type: Error) {
        let mut text = "".to_string();

        let pos = self.lexer.get_sorce_position();

        text.push_str(&format!("{:#?}", self.memo));
        text.push_str(&format!("error type: {:?}\n", error_type));
        text.push_str(&format!("row: {}  column: {}\n", pos.0, pos.1));
        text.push_str(&format!(
            "token pos: {:?}\n",
            self.lexer.get_token_position()
        ));
        text.push_str(&format!("token: {:?}\n", self.lexer.peek()));
        text.push_str(&format!("last wrote memo: {:#?}", self.last_write_memo));

        println!("{}", text);
    }
}
