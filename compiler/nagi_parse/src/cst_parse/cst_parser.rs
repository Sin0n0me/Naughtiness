use crate::cst_parse::expression::*;
use crate::cst_parse::lexer::Lexer;

use nagi_cst::cst::*;
use nagi_cst::keywords::Keyword;
use nagi_cst::token::*;

use std::collections::HashMap;

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

pub struct CSTParser {
    lexer: Lexer,
    memo: HashMap<ParseMemoKey, Option<CSTNode>>,
    min_bp: u16,
    last_write_memo: ParseMemoKey,
}

// TODO 機能ごとの分割
impl CSTParser {
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

    pub fn parse(&mut self) -> Result<CSTNode, Error> {
        let mut node = self.expression()?;

        Ok(node)
    }

    // Expression ::= ExpressionWithoutBlock | ExpressionWithBlock
    fn expression(&mut self) -> Result<CSTNode, Error> {
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

    fn outer_attribute(&mut self) -> Result<CSTNode, Error> {
        Err(Error::E)
    }

    // ExpressionWithoutBlock ::= OuterAttribute*
    //                           (
    //                                LiteralExpression | PathExpression | OperatorExpression | GroupedExpression | ArrayExpression
    //                              | AwaitExpression | IndexExpression | TupleExpression | TupleIndexingExpression | StructExpression
    //                              | CallExpression | MethodCallExpression | FieldExpression | ClosureExpression | AsyncBlockExpression
    //                              | ContinueExpression | BreakExpression | RangeExpression | ReturnExpression | UnderscoreExpression | MacroInvocation
    //                           )
    fn expression_without_block(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("ExpressionWithoutBlock");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        self.outer_attribute();

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

    fn literal_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("LiteralExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        let pos = self.lexer.get_sorce_position();
        let result = match self.lexer.next() {
            Token::Literal(literal) => Ok(CSTNode::new(CSTNodeKind::Literal {
                literal,
                row: pos.0,
                column: pos.1,
            })),

            Token::Keyword(Keyword::True) => Ok(CSTNode::new(CSTNodeKind::Literal {
                literal: Literal::new(LiteralKind::Bool(true), ""),
                row: pos.0,
                column: pos.1,
            })),
            Token::Keyword(Keyword::False) => Ok(CSTNode::new(CSTNodeKind::Literal {
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

    fn path_expression(&mut self) -> Result<CSTNode, Error> {
        Err(Error::E)
    }

    // Pratt parsing
    // OperatorExpression
    fn operator_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("OperatorExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        let min_bp = self.min_bp;

        // 前置演算子
        let mut lhs: CSTNode = match self.lexer.peek() {
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
            println!("op -----> {:?}", op);

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

                let mut node = CSTNode::new(CSTNodeKind::Factor {
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
                let mut node = CSTNode::new(CSTNodeKind::Factor {
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
    fn borrow_expression(&mut self) -> Result<CSTNode, Error> {
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

    fn dereference_expression(&mut self) -> Result<CSTNode, Error> {
        self.expression()
    }

    // GroupedExpression ::= `(` Expression `)`
    fn grouped_expression(&mut self) -> Result<CSTNode, Error> {
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

        Ok(CSTNode::new(CSTNodeKind::GroupedExpression {
            left_parenthesis,
            expression,
            right_parenthesis,
        }))
    }

    // StructExpression ::= StructExprStruct | StructExprTuple | StructExprUnit
    fn struct_expression(&mut self) -> Result<CSTNode, Error> {
        Err(Error::E)
    }

    // StructExprStruct ::= PathInExpression `{` (StructExprFields | StructBase)? `}`

    // StructExprFields ::= StructExprField (, StructExprField)* (, StructBase | ,?)

    // StructExprField  ::= OuterAttribute* ( IDENTIFIER | (IDENTIFIER |TUPLE_INDEX) `:` Expression )

    // StructBase       ::= `..` Expression

    // StructExprTuple  ::=  PathInExpression `(` ( Expression (, Expression)* ,? )? `)`

    // StructExprUnit   ::= PathInExpression

    // CallExpression ::= Expression `(` CallParams? `)`
    fn call_expression(&mut self) -> Result<CSTNode, Error> {
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

        Ok(CSTNode::new(CSTNodeKind::CallExpression {
            expression,
            left_parenthesis,
            call_params,
            right_parenthesis,
        }))
    }

    // CallParams     ::= Expression ( `,` Expression )* `,`?
    fn call_params(&mut self) -> Result<CSTNode, Error> {
        let expression = Box::new(self.expression()?);

        // ( `,` Expression )*
        let mut comma_and_expression = Vec::<(CSTNode, CSTNode)>::new();
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

        Ok(CSTNode::new(CSTNodeKind::CallParams {
            expression,
            comma_and_expression,
            comma,
        }))
    }

    // MethodCallExpression ::= Expression `.` PathExprSegment `(` CallParams? `)`
    fn method_call_expression(&mut self) -> Result<CSTNode, Error> {
        self.expression()?;

        self.call_params();

        Err(Error::E)
    }

    // ReturnExpression ::= return Expression?
    fn return_expression(&mut self) -> Result<CSTNode, Error> {
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

        Ok(CSTNode::new(CSTNodeKind::ReturnExpression {
            return_keyword,
            expression,
        }))
    }

    //IfExpression ::= `if` Expression BlockExpression (`else` ( BlockExpression | IfExpression | IfLetExpression ) )?
    fn if_expression(&mut self) -> Result<CSTNode, Error> {
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
            return Ok(CSTNode::new(CSTNodeKind::Item));
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
    fn if_let_expression(&mut self) -> Result<CSTNode, Error> {
        Err(Error::E)
    }

    // MatchExpression ::= `match` Scrutinee `{` InnerAttribute* MatchArms? `}`
    fn match_expression(&mut self) -> Result<CSTNode, Error> {
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

    fn scrutinee(&mut self) -> Result<CSTNode, Error> {
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
    fn expression_with_block(&mut self) -> Result<CSTNode, Error> {
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
    fn block_expression(&mut self) -> Result<CSTNode, Error> {
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

        let expr = CSTNode::new(CSTNodeKind::BlockExpression {
            left_brace,
            inner_attribute: Vec::new(),
            statements,
            right_brace,
        });
        self.write_memo(&key, Some(&expr));

        Ok(expr)
    }

    // Statements ::= Statement+ | Statement+ ExpressionWithoutBlock | ExpressionWithoutBlock
    fn statements(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Statements");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // Statement+ | Statement+ ExpressionWithoutBlock
        let mut node = CSTNode::new(CSTNodeKind::Statements);
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
    fn statement(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Statement");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        // ;
        if matches!(self.lexer.next(), Token::Semicolon) {
            let expr = CSTNode::new(CSTNodeKind::Statement {
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
            return Ok(CSTNode::new(CSTNodeKind::Statement {
                statement: Box::new(expr),
            }));
        }
        self.backtrack(&key);

        // ExpressionStatement
        if let Ok(expr) = self.expression_statement() {
            self.write_memo(&key, Some(&expr));
            return Ok(CSTNode::new(CSTNodeKind::Statement {
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
    fn let_statement(&mut self) -> Result<CSTNode, Error> {
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

        let expr = CSTNode::new(CSTNodeKind::LetStatement {
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
    fn expression_statement(&mut self) -> Result<CSTNode, Error> {
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
    fn pattern(&mut self) -> Result<CSTNode, Error> {
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
    fn pattern_no_top_alt(&mut self) -> Result<CSTNode, Error> {
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
    fn pattern_without_range(&mut self) -> Result<CSTNode, Error> {
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
    fn literal_pattern(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("LiteralPattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return Err(Error::Recursive),
            MemoResult::None => self.write_memo(&key, None),
        };

        match self.lexer.next() {
            Token::Keyword(keyword) => match keyword {
                Keyword::True => {
                    let node = CSTNode::new(CSTNodeKind::LiteralPattern {
                        literal: Literal::new(LiteralKind::Bool(true), ""),
                    });
                    self.write_memo(&key, Some(&node));
                    Ok(node)
                }

                Keyword::False => {
                    let node = CSTNode::new(CSTNodeKind::LiteralPattern {
                        literal: Literal::new(LiteralKind::Bool(false), ""),
                    });
                    self.write_memo(&key, Some(&node));
                    Ok(node)
                }
                _ => Err(Error::NotExpected),
            },
            Token::Literal(literal) => {
                let node = CSTNode::new(CSTNodeKind::LiteralPattern { literal });
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
                        let node = CSTNode::new(CSTNodeKind::LiteralPattern { literal });
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
    fn identifier_pattern(&mut self) -> Result<CSTNode, Error> {
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

        let node = CSTNode::new(CSTNodeKind::IdentifierPattern {
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
    fn wildcard_pattern(&mut self) -> Result<CSTNode, Error> {
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
    fn rest_pattern(&mut self) -> Result<CSTNode, Error> {
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
    fn reference_pattern(&mut self) -> Result<CSTNode, Error> {
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

    fn make_factor(&self) -> CSTNode {
        let pos = self.lexer.get_sorce_position();
        let token = self.lexer.peek();
        CSTNode::new(CSTNodeKind::Factor {
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

    fn write_memo(&mut self, key: &ParseMemoKey, node: Option<&CSTNode>) {
        if node.is_some() {
            println!(
                "created: {} rule: {} next: {:?}",
                key.position,
                key.rule,
                self.lexer.peek()
            );
        } else {
            println!(
                "write: {} rule: {} token: {:?}",
                key.position,
                key.rule,
                self.lexer.peek()
            );
        }
        self.last_write_memo = key.clone();
        self.memo.insert(key.clone(), node.cloned());
    }

    fn get_memo(&self, key: &ParseMemoKey) -> MemoResult<CSTNode> {
        let Some(node) = self.memo.get(&key) else {
            return MemoResult::None;
        };

        let Some(value) = node.clone() else {
            println!(
                "recursed! {} rule: {}",
                self.lexer.get_token_position(),
                key.rule
            );
            return MemoResult::Recursive;
        };

        MemoResult::Some(value)
    }

    fn backtrack(&mut self, key: &ParseMemoKey) {
        println!(
            "backtrack : {} {} token: {:?}",
            self.lexer.get_token_position(),
            key.rule,
            self.lexer.peek()
        );
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
