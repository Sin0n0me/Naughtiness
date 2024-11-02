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

pub enum Error {
    E,
    NoneMemo,
    Unintended,  // 意図しないトークンだった
    NotExpected, // 期待したトークンではなかった
}

pub struct CSTParser {
    lexer: Lexer,
    memo: HashMap<ParseMemoKey, Option<CSTNode>>,
    stack: Vec<usize>,
    min_bp: u16,
}

// TODO 機能ごとの分割
impl CSTParser {
    pub fn new(token_list: &Vec<nagi_lexer::Token>) -> Self {
        Self {
            lexer: Lexer::new(token_list),
            memo: HashMap::new(),
            stack: Vec::new(),
            min_bp: 0,
        }
    }

    pub fn parse(&mut self) -> Result<CSTNode, Error> {
        self.expression()
    }

    // Expression ::= ExpressionWithoutBlock | ExpressionWithBlock
    fn expression(&mut self) -> Result<CSTNode, Error> {
        if let Some(res) = self.get_memo("Expression") {
            return Ok(res);
        }
        let key = self.make_key("Expression");
        self.write_memo(&key, None);
        self.save();

        if let Ok(res) = self.expression_without_block() {
            self.write_memo(&key, Some(&res));
            return Ok(CSTNode::new(CSTNodeKind::Expression {
                expression: Box::new(res),
            }));
        }

        self.backtrack();
        if let Ok(res) = self.expression_with_block() {
            self.write_memo(&key, Some(&res));
            return Ok(CSTNode::new(CSTNodeKind::Expression {
                expression: Box::new(res),
            }));
        }

        Err(Error::E)
    }

    fn outer_attribute(&mut self) -> Result<CSTNode, Error> {
        Err(Error::E)
    }

    // ExpressionWithoutBlock ::= OuterAttribute*
    //                           (
    //                                LiteralExpression | PathExpression | GroupedExpression | ArrayExpression
    //                              | AwaitExpression | IndexExpression | TupleExpression | TupleIndexingExpression | StructExpression
    //                              | CallExpression | MethodCallExpression | FieldExpression | ClosureExpression | AsyncBlockExpression
    //                              | ContinueExpression | BreakExpression | RangeExpression | ReturnExpression | UnderscoreExpression | MacroInvocation
    //                           )
    fn expression_without_block(&mut self) -> Result<CSTNode, Error> {
        if let Some(res) = self.get_memo("ExpressionWithOutBlock") {
            return Ok(res);
        }
        let key = self.make_key("ExpressionWithOutBlock");
        self.write_memo(&key, None);

        //self.outer_attribute();

        match self.lexer.peek() {
            Token::Literal(literal) => match literal.literal_kind {
                LiteralKind::Integer | LiteralKind::Float => {
                    return self.expression_bp(self.min_bp)
                }
                _ => (),
            },
            Token::Identifier(identifier) => return self.expression_bp(self.min_bp),

            _ => (),
        };

        self.expression_bp(self.min_bp)?;

        Err(Error::E)
    }

    fn literal_expression(&mut self) -> Result<CSTNode, Error> {
        Err(Error::E)
    }

    fn path_expression(&mut self) -> Result<CSTNode, Error> {
        Err(Error::E)
    }

    fn operator_expression(&mut self) -> Result<CSTNode, Error> {
        Err(Error::E)
    }

    // BorrowExpression ::= (`&`|`&&`) Expression
    //                    | (`&`|`&&`) `mut` Expression
    //                    | (`&`|`&&`) `raw` `const` Expression
    //                    | (`&`|`&&`) `raw` `mut` Expression
    fn borrow_expression(&mut self) -> Result<CSTNode, Error> {
        if let Some(node) = self.get_memo("BorrowExpression") {
            return Ok(node);
        }

        self.write_memo(&self.make_key("BorrowExpression"), None);
        if matches!(self.lexer.next(), Token::And | Token::AndAnd) {
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

    // Pratt parsing
    fn expression_bp(&mut self, min_bp: u16) -> Result<CSTNode, Error> {
        let token_pos = self.lexer.get_sorce_position();
        let is_op = self.lexer.is_operator();
        let token = self.lexer.next();

        // 前置演算子
        let mut lhs: CSTNode = match &token {
            Token::Literal(_) | Token::Identifier(_) => CSTNode::new(CSTNodeKind::Factor {
                token: token,
                row: token_pos.0,
                column: token_pos.1,
            }),
            Token::LeftParenthesis(parenthesis) => match &parenthesis {
                LeftParenthesis::Parenthesis => {
                    self.min_bp = 0;
                    let rhs = self.expression()?;
                    let mut node = CSTNode::new(CSTNodeKind::Factor {
                        token,
                        row: token_pos.0,
                        column: token_pos.1,
                    });

                    node.children.push(rhs);

                    node
                }
                _ => return Err(Error::E),
            },
            _ if is_op => {
                let op = self.lexer.peek_glue();
                let Some(((), right_bp)) = prefix_binding_power(&op) else {
                    return Err(Error::E);
                };

                self.min_bp = right_bp; // 次の再帰のために保存
                let rhs = self.expression()?;
                let mut node = CSTNode::new(CSTNodeKind::Factor {
                    token,
                    row: token_pos.0,
                    column: token_pos.1,
                });

                node.children.push(rhs);

                node
            }
            _ => return Err(Error::E),
        };

        loop {
            let op_pos = self.lexer.get_sorce_position();
            let op = self.lexer.peek();
            if matches!(op, Token::Eof) {
                break;
            }

            // 後置演算子
            if let Some((left_bp, ())) = postfix_binding_power(&op) {
                if left_bp < self.min_bp {
                    break;
                }
                self.lexer.next();

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

                self.lexer.next();

                self.min_bp = right_bp; // 次の再帰のために保存
                let rhs = self.expression()?;
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

        Ok(lhs)
    }

    // ReturnExpression ::= return (Expression)?
    fn return_expression(&mut self) -> Result<CSTNode, Error> {
        // `return`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Return)) {
            return Err(Error::E);
        }
        let return_keyword = Box::new(self.make_factor());
        self.lexer.next();

        let mut expression = None;
        if let Ok(expr) = self.expression() {
            expression = Some(Box::new(expr));
        }

        Ok(CSTNode::new(CSTNodeKind::ReturnExpression(
            return_keyword,
            expression,
        )))
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

    // IfLetExpression ::= `if` `let` Pattern `=` Scrutinee BlockExpression (else ( BlockExpression
    // | IfExpression | IfLetExpression ) )?
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
        self.block_expression()
    }

    // BlockExpression ::=  `{` InnerAttribute* Statements? `}`
    fn block_expression(&mut self) -> Result<CSTNode, Error> {
        if let Some(res) = self.get_memo("BlockExpression") {
            return Ok(res);
        }
        let key = self.make_key("BlockExpression");
        self.write_memo(&key, None);
        self.save();

        self.statements()
    }

    // Statements ::= Statement+ | Statement+ ExpressionWithoutBlock | ExpressionWithoutBlock
    fn statements(&mut self) -> Result<CSTNode, Error> {
        let mut node = CSTNode::new(CSTNodeKind::Statements);
        if let Ok(res) = self.statement() {
            node.children.push(res);
        }

        if let Ok(res) = self.expression_without_block() {
            node.children.push(res);
        }

        if node.children.is_empty() {
            Err(Error::E)
        } else {
            Ok(node)
        }
    }

    //
    // Statement
    //

    // Statement ::= `;` | Item | LetStatement | ExpressionStatement | MacroInvocationSemi
    fn statement(&mut self) -> Result<CSTNode, Error> {
        // ;
        if matches!(self.lexer.peek(), Token::Semicolon) {
            return Ok(CSTNode::new(CSTNodeKind::Statement {
                statement: Box::new(self.make_factor()),
            }));
        }

        // Item

        // LetStatement
        if let Ok(res) = self.let_statement() {
            return Ok(CSTNode::new(CSTNodeKind::Statement {
                statement: Box::new(res),
            }));
        }

        // ExpressionStatement
        if let Ok(res) = self.expression_statement() {
            return Ok(CSTNode::new(CSTNodeKind::Statement {
                statement: Box::new(res),
            }));
        }

        // MacroInvocationSemi

        Err(Error::E)
    }

    // LetStatement ::= OuterAttribute* (`ur` | `sr` | `nr` | `let`)
    //                  PatternNoTopAlt ( `:` Type )?
    //                  (`=` Expression ( `else` BlockExpression)? )? `;`
    fn let_statement(&mut self) -> Result<CSTNode, Error> {
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
                _ => return Err(Error::E),
            },
            _ => return Err(Error::E),
        };

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

                colon = Some(Box::new(self.make_factor()));
                self.lexer.next();

                // BlockExpression
                block_expression = Some(Box::new(self.block_expression()?))
            }
        }

        // ;
        if !matches!(self.lexer.peek(), Token::Semicolon) {
            return Err(Error::E);
        }
        let semicolon = Box::new(self.make_factor());

        Ok(CSTNode::new(CSTNodeKind::LetStatement {
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
        }))
    }

    // ExpressionStatement ::= ExpressionWithoutBlock `;` | ExpressionWithBlock `;`?
    fn expression_statement(&mut self) -> Result<CSTNode, Error> {
        if let Ok(res) = self.expression_without_block() {
            if !matches!(self.lexer.peek(), Token::Semicolon) {
                return Err(Error::NotExpected);
            }
            self.lexer.next();

            return Ok(res);
        }

        if let Ok(res) = self.expression_with_block() {
            return Ok(res);
        }

        Err(Error::E)
    }

    //
    // 以下Pattern
    //

    // Pattern ::= `|`? PatternNoTopAlt ( `|` PatternNoTopAlt )*
    fn pattern(&mut self) -> Result<CSTNode, Error> {
        self.pattern_no_top_alt()?;

        self.pattern_no_top_alt()
    }

    // PatternNoTopAlt ::= PatternWithoutRange | RangePattern
    fn pattern_no_top_alt(&mut self) -> Result<CSTNode, Error> {
        self.pattern_without_range()
    }

    // PatternWithoutRange ::= LiteralPattern | IdentifierPattern | WildcardPattern | RestPattern |
    //                         ReferencePattern | StructPattern | TupleStructPattern | TuplePattern | GroupedPattern |
    //                         SlicePattern | PathPattern | MacroInvocation
    fn pattern_without_range(&mut self) -> Result<CSTNode, Error> {
        self.save();

        if let Ok(res) = self.literal_pattern() {
            return Ok(res);
        }

        self.backtrack();
        self.save();
        if let Ok(res) = self.identifier_pattern() {
            return Ok(res);
        }

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
        let res = match self.lexer.next() {
            Token::Literal(literal) => CSTNode::new(CSTNodeKind::LiteralPattern { literal }),

            //
            Token::Minus => match self.lexer.next() {
                Token::Literal(literal) => match literal.literal_kind {
                    LiteralKind::Integer | LiteralKind::Float => {
                        CSTNode::new(CSTNodeKind::LiteralPattern { literal })
                    }
                    _ => return Err(Error::E),
                },
                _ => return Err(Error::E),
            },

            _ => return Err(Error::E),
        };

        Ok(res)
    }

    // IdentifierPattern ::= `ref`? `mut`? Identifier (`@` PatternNoTopAlt )?
    fn identifier_pattern(&mut self) -> Result<CSTNode, Error> {
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

        Ok(CSTNode::new(CSTNodeKind::IdentifierPattern {
            ref_keyword,
            mut_keyword,
            identifier,
            at_symbol,
            pattern_no_top_alt,
        }))
    }

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

    fn save(&mut self) {
        self.stack.push(self.lexer.get_token_position());
    }

    fn backtrack(&mut self) {
        let Some(pos) = self.stack.pop() else {
            return;
        };

        self.lexer.set_postion(pos);
    }

    fn make_key(&self, rule: &str) -> ParseMemoKey {
        ParseMemoKey {
            position: self.lexer.get_token_position(),
            rule: rule.to_string(),
        }
    }

    fn write_memo(&mut self, key: &ParseMemoKey, node: Option<&CSTNode>) {
        self.memo.insert(key.clone(), node.cloned());
    }

    fn get_memo(&self, rule: &str) -> Option<CSTNode> {
        let key = ParseMemoKey {
            position: self.lexer.get_token_position(),
            rule: rule.to_string(),
        };

        let Some(node) = self.memo.get(&key) else {
            return None;
        };

        node.clone()
    }
}
