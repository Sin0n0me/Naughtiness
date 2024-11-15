use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

use crate::expression::*;
use crate::lexer::Lexer;
use crate::parser::*;

use nagi_errors::{Error, ErrorKind, SyntaxError};
use nagi_syntax_tree::cst::*;
use nagi_syntax_tree::keywords::Keyword;
use nagi_syntax_tree::token::*;

pub struct CSTParser {
    log: String,
    lexer: Lexer,
    memo: HashMap<ParseMemoKey, Option<ParseMemoValue<CSTNode>>>,
    min_bp: u16,
}

// TODO 機能ごとの分割
impl CSTParser {
    pub fn new(token_list: &Vec<nagi_lexer::Token>) -> Self {
        Self {
            log: "".to_string(),
            lexer: Lexer::new(token_list),
            memo: HashMap::new(),
            min_bp: 0,
        }
    }

    pub fn parse(&mut self) -> Result<CSTNode, Error> {
        self.crates_and_source_files()
    }

    fn crates_and_source_files(&mut self) -> Result<CSTNode, Error> {
        let mut inner_attributes = Vec::<CSTNode>::new();
        let mut items = Vec::<CSTNode>::new();

        // InnerAttribute*
        while let Ok(inner_attribute) = self.inner_attribute() {
            inner_attributes.push(inner_attribute);
        }
        // Item*
        while let Ok(item) = self.item() {
            items.push(item);
        }

        //println!("{:#?}", self.memo);

        if matches!(self.lexer.peek(), Token::Eof) {
            self.log.push_str("Parse success\n");

            Ok(CSTNode::new(
                CSTNodeKind::Crate {
                    inner_attributes,
                    items,
                },
                vec![],
            ))
        } else {
            self.log.push_str("Parse error\n");
            Err(Error {
                error_kind: ErrorKind::Syntax(SyntaxError::NotMatch),
                error_text: "".to_string(),
            })
        }
    }

    //
    // Attributes
    //

    // InnerAttribute ::= `#` `!` `[` Attribute `]`
    fn inner_attribute(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("InnerAttribute");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // `#`
        if !matches!(self.lexer.peek(), Token::Pound) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let pound = Box::new(self.make_factor_and_next());

        // `!`
        if !matches!(self.lexer.peek(), Token::Not) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let exclamation = Box::new(self.make_factor_and_next());

        // `[`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Brackets)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let left_brackets = Box::new(self.make_factor_and_next());

        // Attribute
        let attribute = Box::new(self.attribute()?);

        // `]`
        if !matches!(
            self.lexer.peek(),
            Token::RightParenthesis(RightParenthesis::Brackets)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let right_brackets = Box::new(self.make_factor_and_next());

        let node = CSTNode::new(
            CSTNodeKind::InnerAttribute {
                pound,
                exclamation,
                left_brackets,
                attribute,
                right_brackets,
            },
            vec![],
        );
        self.write_memo(&key, Some(&node));

        Ok(node)
    }

    // OuterAttribute ::= `#` `[` Attribute `]`
    fn outer_attribute(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("OuterAttribute");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // `#`
        if !matches!(self.lexer.peek(), Token::Pound) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let pound = Box::new(self.make_factor_and_next());

        // `[`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Brackets)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let left_brackets = Box::new(self.make_factor_and_next());

        let attribute = Box::new(self.attribute()?);

        // `]`
        if !matches!(
            self.lexer.peek(),
            Token::RightParenthesis(RightParenthesis::Brackets)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let right_brackets = Box::new(self.make_factor_and_next());

        let node = CSTNode::new(
            CSTNodeKind::OuterAttribute {
                pound,
                left_brackets,
                attribute,
                right_brackets,
            },
            vec![],
        );
        self.write_memo(&key, Some(&node));

        Ok(node)
    }

    // Attribute ::= SimplePath AttributeInput?  | `unsafe` `(` SimplePath AttributeInput? `)`
    fn attribute(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Attribute");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        self.error(SyntaxError::NotMatch, &key)
    }

    // AttributeInput ::= DelimTokenTree | `=` Expression
    fn attribute_input(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("AttributeInput");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        self.error(SyntaxError::NotMatch, &key)
    }

    // Visibility ::= `pub`
    //              | `pub` `(` `crate` `)`
    //              | `pub` `(` `self` `)`
    //              | `pub` `(` `super` `)`
    //              | `pub` `(` `in` SimplePath `)`
    fn visibility(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Visibility");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // TODO

        // `pub`
        let mut pub_keyword = CSTNode::new(
            CSTNodeKind::Visibility {
                pub_keyword: Box::new(self.make_factor()),
            },
            vec![],
        );

        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Pub)) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }

        if matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return Ok(pub_keyword);
        }

        Ok(pub_keyword)
    }

    //
    // Items
    //

    // Item ::= OuterAttribute* VisItem | MacroItem
    fn item(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Item");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // OuterAttribute*
        let mut outer_attribute = Vec::<CSTNode>::new();
        while let Ok(expr) = self.outer_attribute() {
            outer_attribute.push(expr);
        }

        self.vis_item()

        // TODO
    }

    // VisItem ::= Visibility?
    //           (
    //             Module
    //           | ExternCrate
    //           | UseDeclaration
    //           | Function
    //           | TypeAlias
    //           | Struct
    //           | Enumeration
    //           | Union
    //           | ConstantItem
    //           | StaticItem
    //           | Trait
    //           | Implementation
    //           | ExternBlock
    //           )
    fn vis_item(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("VisItem");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // Visibility?
        let mut visibility = None;
        if let Ok(expr) = self.visibility() {
            visibility = Some(Box::new(expr));
        }

        // Module
        // ExternCrate
        // UseDeclaration

        // Function
        if let Ok(expr) = self.function() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        // TypeAlias
        // Struct
        // Enumeration
        // Union
        // ConstantItem
        // StaticItem
        // Trait
        // Implementation
        // ExternBlock

        self.error(SyntaxError::NotMatch, &key)
    }

    //
    // Functions
    //

    // Function ::= FunctionQualifiers `fn` Identifier GenericParams?
    //             `(` FunctionParameters? `)`
    //             FunctionReturnType? WhereClause?
    //             ( BlockExpression | `;` )
    fn function(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Function");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // FunctionQualifiers
        let function_qualifiers = Box::new(self.function_qualifiers()?);

        // `fn`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Fn)) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let fn_keyword = Box::new(self.make_factor_and_next());

        // Identifier
        if !matches!(self.lexer.peek(), Token::Identifier(_)) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let identifier = Box::new(self.make_factor_and_next());

        // GenericParams?
        let mut generic_params = None;
        if let Ok(expr) = self.generic_params() {
            generic_params = Some(Box::new(expr));
        }

        // `(`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let left_parenthesis = Box::new(self.make_factor_and_next());

        // FunctionParameters?
        let mut function_parameters = None;
        if let Ok(param) = self.function_parameters() {
            function_parameters = Some(Box::new(param));
        }

        // `)`
        if !matches!(
            self.lexer.peek(),
            Token::RightParenthesis(RightParenthesis::Parenthesis)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let right_parenthesis = Box::new(self.make_factor_and_next());

        // FunctionReturnType?
        let mut function_return_type = None;
        if let Ok(expr) = self.function_return_type() {
            function_return_type = Some(Box::new(expr));
        }

        // WhereClause?
        let mut where_clause = None;

        // ( BlockExpression | `;` )
        if let Token::Semicolon = self.lexer.peek() {
            let node = CSTNode::new(
                CSTNodeKind::Function {
                    function_qualifiers,
                    fn_keyword,
                    identifier,
                    generic_params,
                    left_parenthesis,
                    function_parameters,
                    right_parenthesis,
                    function_return_type,
                    where_clause,
                    block_expression_or_semicolon: Box::new(self.make_factor_and_next()),
                },
                vec![],
            );

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        if let Ok(expr) = self.block_expression() {
            let node = CSTNode::new(
                CSTNodeKind::Function {
                    function_qualifiers,
                    fn_keyword,
                    identifier,
                    generic_params,
                    left_parenthesis,
                    function_parameters,
                    right_parenthesis,
                    function_return_type,
                    where_clause,
                    block_expression_or_semicolon: Box::new(expr),
                },
                vec![],
            );

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // FunctionQualifiers ::= `const`? `async`? ItemSafety? (`extern` Abi?)?
    fn function_qualifiers(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("FunctionQualifiers");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        let mut const_keyword = None;
        let mut async_keyword = None;
        let mut item_safety = None;
        let mut extern_keyword = None;
        let mut abi = None;

        // `const`?
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Const)) {
            const_keyword = Some(Box::new(self.make_factor_and_next()));
        }

        // `async`?
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Async)) {
            async_keyword = Some(Box::new(self.make_factor_and_next()));
        }

        // ItemSafety?
        if let Ok(expr) = self.item_safety() {
            item_safety = Some(Box::new(expr));
        }

        // (`extern` `Abi`?)?
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Extern)) {
            extern_keyword = Some(Box::new(self.make_factor_and_next()));

            if let Ok(expr) = self.abi() {
                abi = Some(Box::new(expr));
            }
        }

        let node = CSTNode::new(
            CSTNodeKind::FunctionQualifiers {
                const_keyword,
                async_keyword,
                item_safety,
                extern_keyword,
                abi,
            },
            vec![],
        );
        self.write_memo(&key, Some(&node));

        Ok(node)
    }

    // ItemSafety ::= `safe` | `unsafe`
    fn item_safety(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("ItemSafety");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        match self.lexer.peek() {
            Token::Keyword(Keyword::Unsafe) => Ok(self.make_factor_and_next()),
            Token::Identifier(identifier) => {
                if identifier != "safe" {
                    return self.error(SyntaxError::ExpectedToken, &key);
                }
                Ok(self.make_factor_and_next())
            }
            _ => self.error(SyntaxError::ExpectedToken, &key),
        }
    }

    // Abi ::= STRING_LITERAL | RAW_STRING_LITERAL
    fn abi(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Abi");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        match self.lexer.peek() {
            Token::Literal(literal) => {
                if !matches!(literal.literal_kind, LiteralKind::Str | LiteralKind::StrRaw) {
                    return self.error(SyntaxError::ExpectedToken, &key);
                }

                Ok(self.make_factor_and_next())
            }
            _ => self.error(SyntaxError::ExpectedToken, &key),
        }
    }

    // GenericParams ::= `<` `>` | `<` (GenericParam `,`)* GenericParam `,`? `>`
    fn generic_params(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("GenericParams");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // `<` `>`
        if !matches!(self.lexer.peek(), Token::LessThan) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        self.make_factor_and_next();
        if matches!(self.lexer.peek(), Token::GreaterThan) {
            return Ok(self.make_factor_and_next());
        }

        // (GenericParam `,`)* GenericParam `,`?
        // TODO

        self.error(SyntaxError::NotMatch, &key)
    }

    // FunctionParameters ::= SelfParam `,`? | (SelfParam `,`)? FunctionParam (`,` FunctionParam)* `,`?
    fn function_parameters(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("FunctionParameters");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // SelfParam
        let first_self_param = self.self_param();

        // `,`
        let commna = if matches!(self.lexer.peek(), Token::Comma) {
            Some(self.make_factor_and_next())
        } else {
            None
        };

        // SelfParamと,両方存在すればFunctionparamの判定へ
        if first_self_param.is_ok() && commna.is_none() {
            // SelfParamのみ
            let node = CSTNode::new(
                CSTNodeKind::FunctionParam1 {
                    self_param: Box::new(first_self_param.unwrap()),
                    comma: None,
                },
                vec![],
            );
            self.write_memo(&key, Some(&node));

            return Ok(node);
        } else if first_self_param.is_err() && commna.is_some() {
            // ,のみはエラー
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let self_param = if first_self_param.is_ok() && commna.is_some() {
            Some((
                Box::new(first_self_param.unwrap()),
                Box::new(commna.unwrap()),
            ))
        } else {
            None
        };

        // FunctionParam
        let function_param = Box::new(self.function_param()?);

        // (`,` FunctionParam)* `,`?
        let mut function_param_repeat = Vec::<(CSTNode, CSTNode)>::new();
        while let Token::Comma = self.lexer.peek() {
            // `,`
            let comma = self.make_factor_and_next();

            // FunctionParam
            let Ok(param) = self.function_param() else {
                break;
            };

            function_param_repeat.push((comma, param));
        }

        // `,`
        let last_comma = if matches!(self.lexer.peek(), Token::Comma) {
            Some(Box::new(self.make_factor_and_next()))
        } else {
            None
        };

        let node = CSTNode::new(
            CSTNodeKind::FunctionParam2 {
                self_param,
                function_param,
                function_param_repeat,
                comma: last_comma,
            },
            vec![],
        );

        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // SelfParam ::= OuterAttribute* ( ShorthandSelf | TypedSelf )
    fn self_param(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Selfparam");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // OuterAttribute*
        let mut outer_attribute = Vec::<CSTNode>::new();
        while let Ok(expr) = self.outer_attribute() {
            outer_attribute.push(expr);
        }

        // ( ShorthandSelf | TypedSelf )
        if let Ok(expr) = self.shorthand_self() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        } else if let Ok(expr) = self.typed_self() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // ShorthandSelf ::= (`&` | `&` Lifetime)? `mut`? `self`
    fn shorthand_self(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("ShorthandSelf");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

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
            return self.error(SyntaxError::NotMatch, &key);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // TypedSelf ::= `mut`? `self` `:` Type
    fn typed_self(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("TypedSelf");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        let mut node = self.make_factor();

        // `mut`?
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Mut)) {
            // TODO
        }

        // `self`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::SelfValue)) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }

        // `:`
        if !matches!(self.lexer.peek(), Token::Colon) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }

        // Type
        self.type_expression()?;

        Ok(node)
    }

    // FunctionParam ::= OuterAttribute* ( FunctionParamPattern | `...` | Type )
    fn function_param(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("FunctionParam");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // OuterAttribute*
        let mut outer_attribute = Vec::<CSTNode>::new();
        while let Ok(expr) = self.outer_attribute() {
            outer_attribute.push(expr);
        }

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

        self.error(SyntaxError::NotMatch, &key)
    }

    // FunctionParamPattern ::= PatternNoTopAlt `:` ( Type | `...` )
    fn function_param_pattern(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("FunctionParamPattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // PatternNoTopAlt
        self.pattern_no_top_alt()?;

        // `:`
        if !matches!(self.lexer.peek(), Token::Colon) {
            return self.error(SyntaxError::NotMatch, &key);
        }

        // ( Type | `...` )
        match self.lexer.peek_glue() {
            Token::DotDotDot => (),
            _ => (),
        };

        // TODO
        self.error(SyntaxError::NotMatch, &key)
    }

    // FunctionReturnType ::= `->` Type
    fn function_return_type(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("FunctionReturnType");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        self.error(SyntaxError::NotMatch, &key)
    }

    // Structs

    //
    // Type
    //

    // Type ::= TypeNoBounds | ImplTraitType | TraitObjectType
    fn type_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("FunctionReturnType");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        self.error(SyntaxError::NotMatch, &key)
    }

    //
    // Expressions
    //

    // Expression ::= ExpressionWithoutBlock | ExpressionWithBlock
    fn expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Expression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        if let Ok(expr) = self.expression_without_block() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        if let Ok(expr) = self.expression_with_block() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        self.error(SyntaxError::NotMatch, &key)
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
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // OuterAttribute*
        let mut outer_attribute = Vec::<CSTNode>::new();
        while let Ok(expr) = self.outer_attribute() {
            outer_attribute.push(expr);
        }

        // OperatorExpression
        if let Ok(expr) = self.operator_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(CSTNode::new(
                CSTNodeKind::ExpressionWithoutBlock {
                    outer_attribute,
                    expression: Box::new(expr),
                },
                vec![],
            ));
        }

        // LiteralExpression
        if let Ok(expr) = self.literal_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(CSTNode::new(
                CSTNodeKind::ExpressionWithoutBlock {
                    outer_attribute,
                    expression: Box::new(expr),
                },
                vec![],
            ));
        }

        // PathExpression
        if let Ok(expr) = self.path_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(CSTNode::new(
                CSTNodeKind::ExpressionWithoutBlock {
                    outer_attribute,
                    expression: Box::new(expr),
                },
                vec![],
            ));
        }

        // GroupedExpression
        if let Ok(expr) = self.grouped_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(CSTNode::new(
                CSTNodeKind::ExpressionWithoutBlock {
                    outer_attribute,
                    expression: Box::new(expr),
                },
                vec![],
            ));
        }

        // StructExpression
        if let Ok(expr) = self.struct_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(CSTNode::new(
                CSTNodeKind::ExpressionWithoutBlock {
                    outer_attribute,
                    expression: Box::new(expr),
                },
                vec![],
            ));
        }

        // CallExpression
        if let Ok(expr) = self.call_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(CSTNode::new(
                CSTNodeKind::ExpressionWithoutBlock {
                    outer_attribute,
                    expression: Box::new(expr),
                },
                vec![],
            ));
        }

        // MethodCallExpression
        if let Ok(expr) = self.method_call_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(CSTNode::new(
                CSTNodeKind::ExpressionWithoutBlock {
                    outer_attribute,
                    expression: Box::new(expr),
                },
                vec![],
            ));
        }

        // ReturnExpression
        if let Ok(expr) = self.return_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(CSTNode::new(
                CSTNodeKind::ExpressionWithoutBlock {
                    outer_attribute,
                    expression: Box::new(expr),
                },
                vec![],
            ));
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    fn literal_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("LiteralExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        let pos = self.lexer.get_sorce_position();
        let literal = match self.lexer.next() {
            Token::Literal(literal) => CSTNode::new(
                CSTNodeKind::Literal {
                    literal,
                    row: pos.0,
                    column: pos.1,
                },
                vec![],
            ),

            Token::Keyword(Keyword::True) => CSTNode::new(
                CSTNodeKind::Literal {
                    literal: Literal::new(LiteralKind::Bool(true), ""),
                    row: pos.0,
                    column: pos.1,
                },
                vec![],
            ),
            Token::Keyword(Keyword::False) => CSTNode::new(
                CSTNodeKind::Literal {
                    literal: Literal::new(LiteralKind::Bool(false), ""),
                    row: pos.0,
                    column: pos.1,
                },
                vec![],
            ),

            _ => return self.error(SyntaxError::ExpectedToken, &key),
        };

        self.write_memo(&key, Some(&literal));
        Ok(literal)
    }

    // PathExpression ::= PathInExpression | QualifiedPathInExpression
    fn path_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("PathExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // PathInExpression
        if let Ok(expr) = self.path_in_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        // QualifiedPathInExpression
        if let Ok(expr) = self.qualified_path_in_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // PathInExpression ::= `::`? PathExprSegment (`::` PathExprSegment)*
    fn path_in_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("PathInExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        let mut path_separater = None;

        //  `::`?
        if matches!(self.lexer.peek_glue(), Token::PathSeparater) {
            let pos = self.lexer.get_sorce_position();
            path_separater = Some(Box::new(CSTNode::new(
                CSTNodeKind::Factor {
                    token: self.lexer.next_glue(),
                    row: pos.0,
                    column: pos.1,
                },
                vec![],
            )));
        }

        // PathExprSegment
        let path_expr_segment = Box::new(self.path_expr_segment()?);

        // (`::` PathExprSegment)*
        let mut repeat_path_expr_segment = Vec::<(CSTNode, CSTNode)>::new();
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
                CSTNode::new(
                    CSTNodeKind::Factor {
                        token: Token::PathSeparater,
                        row: pos.0,
                        column: pos.1,
                    },
                    vec![],
                ),
                expr,
            ));
        }

        let node = CSTNode::new(
            CSTNodeKind::PathInExpression {
                path_separater,
                path_expr_segment,
                repeat_path_expr_segment,
            },
            vec![],
        );
        self.write_memo(&key, Some(&node));

        Ok(node)
    }

    // PathExprSegment ::= PathIdentSegment (`::` GenericArgs)?
    fn path_expr_segment(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("PathExprSegment");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // PathIdentSegment
        let path_ident_segment = Box::new(self.path_ident_segment()?);

        //  (`::` GenericArgs)?
        let mut generic_args = None;
        if matches!(self.lexer.peek_glue(), Token::PathSeparater) {

            // GenericArgs
        }

        Ok(CSTNode::new(
            CSTNodeKind::PathExprSegment {
                path_ident_segment,
                generic_args,
            },
            vec![],
        ))
    }

    // PathIdentSegment   ::= Identifier | `super` | `self` | `Self` | `crate` | `$crate`
    fn path_ident_segment(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("PathIdentSegment");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
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
                _ => self.error(SyntaxError::NotMatch, &key),
            },

            _ => self.error(SyntaxError::NotMatch, &key),
        }
    }

    // QualifiedPathInExpression ::= QualifiedPathType (`::` PathExprSegment)+
    fn qualified_path_in_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("QualifiedPathInExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // TODO

        // QualifiedPathType
        self.qualified_path_type()?;

        // (`::` PathExprSegment)+

        self.error(SyntaxError::NotMatch, &key)
    }

    // QualifiedPathType ::= `<` Type (`as` TypePath)? `>`
    fn qualified_path_type(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("QualifiedPathType");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        self.error(SyntaxError::NotMatch, &key)
    }

    // QualifiedPathInType ::= QualifiedPathType (`::` TypePathSegment)+
    fn qualified_path_in_type(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("QualifiedPathInType");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        self.qualified_path_type()?;

        self.error(SyntaxError::NotMatch, &key)
    }

    // Pratt parsing
    // OperatorExpression
    fn operator_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("OperatorExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        let min_bp = self.min_bp;

        // 前置演算子
        let mut lhs: CSTNode = if is_operator(&self.lexer.peek_glue()) {
            let op = self.lexer.peek_glue();
            let Some(((), right_bp)) = prefix_binding_power(&op) else {
                return self.error(SyntaxError::ExpectedToken, &key);
            };

            self.min_bp = right_bp; // 次の再帰のために保存
            let mut node = self.make_operator_and_next();
            node.children.push(self.operator_expression()?);

            node
        } else {
            // TODO Expressionの最初に呼び出す
            // Expressionの再帰用に呼び出し元だけを削除
            self.memo.remove(&self.make_key("Expression"));
            self.memo.remove(&self.make_key("ExpressionWithoutBlock"));
            self.expression()?

            /*
            match self.expression() {
                Ok(expr) => expr,
                Err(error) => {
                    self.min_bp = 0;
                    return Err(error);
                }
            }
             * */
        };

        loop {
            let op_pos = self.lexer.get_sorce_position();
            let op = self.lexer.peek_glue();

            if !is_operator(&op) {
                break;
            }

            // 後置演算子
            if let Some((left_bp, ())) = postfix_binding_power(&op) {
                if left_bp < min_bp {
                    break;
                }
                self.lexer.next_glue();

                lhs = CSTNode::new(
                    CSTNodeKind::Operator {
                        token: op,
                        row: op_pos.0,
                        column: op_pos.1,
                    },
                    vec![lhs],
                );

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
                let node = CSTNode::new(
                    CSTNodeKind::Operator {
                        token: op,
                        row: op_pos.0,
                        column: op_pos.1,
                    },
                    vec![lhs, rhs],
                );

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
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // (`&`|`&&`)
        if matches!(self.lexer.peek_glue(), Token::And | Token::AndAnd) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        self.lexer.next_glue();

        if let Token::Keyword(keyword) = self.lexer.peek() {
            if !matches!(keyword, Keyword::Mut) {
                return self.error(SyntaxError::NotMatch, &key);
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
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // TODO

        // `(`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let left_parenthesis = Box::new(self.make_factor_and_next());

        // Expression
        let expression = Box::new(self.expression()?);

        // `)`
        if !matches!(
            self.lexer.peek(),
            Token::RightParenthesis(RightParenthesis::Parenthesis)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let right_parenthesis = Box::new(self.make_factor_and_next());

        Ok(CSTNode::new(
            CSTNodeKind::GroupedExpression {
                left_parenthesis,
                expression,
                right_parenthesis,
            },
            vec![],
        ))
    }

    // StructExpression ::= StructExprStruct | StructExprTuple | StructExprUnit
    fn struct_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("StructExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        if let Ok(expr) = self.struct_expr_struct() {
            self.write_memo(&key, Some(&expr));
            return Ok(CSTNode::new(
                CSTNodeKind::StructExpression {
                    expression: Box::new(expr),
                },
                vec![],
            ));
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // StructExprStruct ::= PathInExpression `{` (StructExprFields | StructBase)? `}`
    fn struct_expr_struct(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("StructExprStruct");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        let path_in_expression = Box::new(self.path_in_expression()?);

        // `{`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Brace)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let left_brace = Box::new(self.make_factor_and_next());

        // (StructExprFields | StructBase)?
        let mut expression = None;
        if let Ok(expr) = self.struct_expr_fileds() {
            expression = Some(Box::new(expr));
        } else if let Ok(expr) = self.struct_expr_filed() {
            expression = Some(Box::new(expr));
        }

        // `}`
        if !matches!(
            self.lexer.peek(),
            Token::RightParenthesis(RightParenthesis::Brace)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let right_brace = Box::new(self.make_factor_and_next());

        let node = CSTNode::new(
            CSTNodeKind::StructExprStruct {
                path_in_expression,
                left_brace,
                expression,
                right_brace,
            },
            vec![],
        );
        self.write_memo(&key, Some(&node));

        Ok(node)
    }

    // StructExprFields ::= StructExprField (, StructExprField)* (, StructBase | ,?)
    fn struct_expr_fileds(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("StructExprFields");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // StructExprField
        let struct_expr_filed = Box::new(self.struct_expr_filed()?);

        // (, StructExprField)*
        let mut struct_expr_filed_repeat = Vec::<(CSTNode, CSTNode)>::new();
        loop {
            // `,`
            if !matches!(self.lexer.peek(), Token::Comma) {
                break;
            }
            let comma = self.make_factor_and_next();

            let Ok(expr) = self.struct_expr_filed() else {
                break;
            };

            struct_expr_filed_repeat.push((comma, expr));
        }

        // (, StructBase | ,?)
        let mut comma = None;
        let mut struct_base = None;
        if matches!(self.lexer.peek(), Token::Comma) {
            comma = Some(Box::new(self.make_factor_and_next()));
            if let Ok(expr) = self.struct_base() {
                struct_base = Some(Box::new(expr));
            }
        }

        let node = CSTNode::new(
            CSTNodeKind::StructExprFields {
                struct_expr_filed,
                struct_expr_filed_repeat,
                comma,
                struct_base,
            },
            vec![],
        );
        self.write_memo(&key, Some(&node));

        Ok(node)
    }

    // StructExprField  ::= OuterAttribute* ( Identifier | (Identifier |TUPLE_INDEX) `:` Expression )
    fn struct_expr_filed(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("StructExprField");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // OuterAttribute*
        let mut outer_attribute = Vec::<CSTNode>::new();
        while let Ok(expr) = self.outer_attribute() {
            outer_attribute.push(expr);
        }

        // Identifier
        let mut is_identifier = false;
        let identifier_or_tuple = match self.lexer.peek() {
            Token::Identifier(_) => {
                is_identifier = true;
                Box::new(self.make_factor_and_next())
            }
            Token::Literal(literal) => match literal.literal_kind {
                LiteralKind::Integer => Box::new(self.make_factor_and_next()),
                _ => return self.error(SyntaxError::NotMatch, &key),
            },
            _ => return self.error(SyntaxError::ExpectedToken, &key),
        };

        Token::Literal(Literal::new(LiteralKind::Integer, "0"));

        // `:`
        if !matches!(self.lexer.peek(), Token::Colon) {
            if !is_identifier {
                return self.error(SyntaxError::NotMatch, &key);
            }

            let node = CSTNode::new(
                CSTNodeKind::StructExprField1 {
                    outer_attribute,
                    identifier: identifier_or_tuple,
                },
                vec![],
            );

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }
        let colon = Box::new(self.make_factor_and_next());

        // Expression
        let expression = Box::new(self.expression()?);

        let node = CSTNode::new(
            CSTNodeKind::StructExprField2 {
                outer_attribute,
                identifier_or_tuple,
                colon,
                expression,
            },
            vec![],
        );

        self.write_memo(&key, Some(&node));
        return Ok(node);
    }

    // StructBase ::= `..` Expression
    fn struct_base(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("StructBase");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // `..`
        if !matches!(self.lexer.peek(), Token::DotDot) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let dotdot = Box::new(self.make_factor_and_next());

        // Expression
        let expression = Box::new(self.expression()?);

        let node = CSTNode::new(CSTNodeKind::StructBase { dotdot, expression }, vec![]);
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // StructExprTuple  ::=  PathInExpression `(` ( Expression (, Expression)* ,? )? `)`

    // StructExprUnit   ::= PathInExpression

    // CallExpression ::= Expression `(` CallParams? `)`
    fn call_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("CallExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // Expression
        let expression = Box::new(self.expression()?);

        // `(`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let left_parenthesis = Box::new(self.make_factor_and_next());

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
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let right_parenthesis = Box::new(self.make_factor_and_next());

        Ok(CSTNode::new(
            CSTNodeKind::CallExpression {
                expression,
                left_parenthesis,
                call_params,
                right_parenthesis,
            },
            vec![],
        ))
    }

    // CallParams ::= Expression ( `,` Expression )* `,`?
    fn call_params(&mut self) -> Result<CSTNode, Error> {
        let expression = Box::new(self.expression()?);

        // ( `,` Expression )*
        let mut comma_and_expression = Vec::<(CSTNode, CSTNode)>::new();
        loop {
            if !matches!(self.lexer.peek(), Token::Comma) {
                break;
            }
            let comma = self.make_factor_and_next();

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

        Ok(CSTNode::new(
            CSTNodeKind::CallParams {
                expression,
                comma_and_expression,
                comma,
            },
            vec![],
        ))
    }

    // MethodCallExpression ::= Expression `.` PathExprSegment `(` CallParams? `)`
    fn method_call_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("MethodCallExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        self.expression()?;

        self.call_params();

        self.error(SyntaxError::NotMatch, &key)
    }

    // ReturnExpression ::= `return` Expression?
    fn return_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("ReturnExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        // `return`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Return)) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        let return_keyword = Box::new(self.make_factor_and_next());

        // Expression?
        let mut expression = None;
        if let Ok(expr) = self.expression() {
            expression = Some(Box::new(expr));
        }

        Ok(CSTNode::new(
            CSTNodeKind::ReturnExpression {
                return_keyword,
                expression,
            },
            vec![],
        ))
    }

    // Scrutinee ::= Expression
    // ** except struct expression **
    fn scrutinee(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Scrutinee");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        let Ok(expression) = self.expression() else {
            return self.error(SyntaxError::NotMatch, &key);
        };

        match expression.node_kind {
            CSTNodeKind::StructExpression { expression } => {
                self.error(SyntaxError::ExpectedToken, &key)
            }
            _ => return Ok(expression),
        }
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
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        // OuterAttribute*
        let mut outer_attribute = Vec::<CSTNode>::new();
        while let Ok(expr) = self.outer_attribute() {
            outer_attribute.push(expr);
        }

        // BlockExpression
        if let Ok(expr) = self.block_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        // IfExpression
        if let Ok(expr) = self.if_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        // IfLetExpression
        if let Ok(expr) = self.if_let_expression() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // BlockExpression ::=  `{` InnerAttribute* Statements? `}`
    fn block_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("BlockExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // `{`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Brace)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let left_brace = Box::new(self.make_factor_and_next());

        // Statements?
        let mut statements = None;
        if let Ok(expr) = self.statements() {
            statements = Some(Box::new(expr));
        }

        // `}`
        if !matches!(
            self.lexer.peek(),
            Token::RightParenthesis(RightParenthesis::Brace)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let right_brace = Box::new(self.make_factor_and_next());

        let expr = CSTNode::new(
            CSTNodeKind::BlockExpression {
                left_brace,
                inner_attribute: Vec::new(),
                statements,
                right_brace,
            },
            vec![],
        );
        self.write_memo(&key, Some(&expr));

        Ok(expr)
    }

    // Statements ::= Statement+ | Statement+ ExpressionWithoutBlock | ExpressionWithoutBlock
    fn statements(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Statements");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // Statement+ | Statement+ ExpressionWithoutBlock
        let mut node = CSTNode::new(CSTNodeKind::Statements, vec![]);
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

        self.error(SyntaxError::NotMatch, &key)
    }

    // ConstBlockExpression ::= `const` BlockExpression
    fn const_block_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("ConstBlockExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // `const`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Const)) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        let const_keyword = Box::new(self.make_factor_and_next());

        // TODO

        self.error(SyntaxError::NotMatch, &key)
    }

    // UnsafeBlockExpression ::= `unsafe` BlockExpression

    //IfExpression ::= `if` Expression BlockExpression (`else` ( BlockExpression | IfExpression | IfLetExpression ) )?
    fn if_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("IfExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        // `if`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::If)) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        let if_keyword = Box::new(self.make_factor_and_next());

        // Expression
        let expression = Box::new(self.expression()?);

        // BlockExpression
        let block_expression = Box::new(self.block_expression()?);

        // ( `else` ( BlockExpression | IfExpression | IfLetExpression ) )?
        let mut else_keyword = None;
        let mut else_expression = None;
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Else)) {
            else_keyword = Some(Box::new(self.make_factor_and_next()));

            // ( BlockExpression | IfExpression | IfLetExpression )
            if let Ok(expr) = self.block_expression() {
                else_expression = Some(Box::new(expr));
            } else if let Ok(expr) = self.if_expression() {
                else_expression = Some(Box::new(expr));
            } else if let Ok(expr) = self.if_let_expression() {
                else_expression = Some(Box::new(expr));
            } else {
                return self.error(SyntaxError::ExpectedToken, &key);
            }
        }

        let node = CSTNode::new(
            CSTNodeKind::IfExpression {
                if_keyword,
                expression,
                block_expression,
                else_keyword,
                else_expression,
            },
            vec![],
        );
        self.write_memo(&key, Some(&node));

        Ok(node)
    }

    // IfLetExpression ::= `if` `let` Pattern `=` Scrutinee BlockExpression
    //                   ( else ( BlockExpression | IfExpression | IfLetExpression ) )?
    fn if_let_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("IfLetExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // `if`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::If)) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        let if_keyword = Box::new(self.make_factor_and_next());

        // `let`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Let)) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        let let_keyword = Box::new(self.make_factor_and_next());

        // Pattern
        let pattern = Box::new(self.pattern()?);

        // =
        if !matches!(self.lexer.peek(), Token::Equal) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        let equal = Box::new(self.make_factor_and_next());

        // Scrutinee
        // ** except lazy boolean operator expression **
        let scrutinee = Box::new(self.scrutinee()?);

        // BlockExpression
        let block_expression = Box::new(self.block_expression()?);

        // ( else ( BlockExpression | IfExpression | IfLetExpression ) )?
        let mut else_keyword = None;
        let mut else_expression = None;
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Else)) {
            else_keyword = Some(Box::new(self.make_factor_and_next()));

            // ( BlockExpression | IfExpression | IfLetExpression )
            if let Ok(expr) = self.block_expression() {
                else_expression = Some(Box::new(expr));
            } else if let Ok(expr) = self.if_expression() {
                else_expression = Some(Box::new(expr));
            } else if let Ok(expr) = self.if_let_expression() {
                else_expression = Some(Box::new(expr));
            } else {
                return self.error(SyntaxError::ExpectedToken, &key);
            }
        }

        let node = CSTNode::new(
            CSTNodeKind::IfLetExpression {
                if_keyword,
                let_keyword,
                pattern,
                equal,
                scrutinee,
                block_expression,
                else_keyword,
                else_expression,
            },
            vec![],
        );
        self.write_memo(&key, Some(&node));

        Ok(node)
    }

    // MatchExpression ::= `match` Scrutinee `{` InnerAttribute* MatchArms? `}`
    fn match_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("MatchExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        // `match`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Match)) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        self.make_factor_and_next();

        // Scrutinee
        let scrutinee = self.scrutinee()?;

        // `{`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Brace)
        ) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        self.make_factor_and_next();

        // InnerAttribute*
        loop {
            let Ok(inner_attribute) = self.inner_attribute() else {
                break;
            };
        }

        // MatchArms?

        // `}`
        if !matches!(
            self.lexer.peek(),
            Token::RightParenthesis(RightParenthesis::Brace)
        ) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        self.make_factor_and_next();

        self.error(SyntaxError::NotMatch, &key)
    }

    //
    // Statement
    //

    // Statement ::= `;` | Item | LetStatement | ExpressionStatement | MacroInvocationSemi
    fn statement(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Statement");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // ;
        if matches!(self.lexer.peek(), Token::Semicolon) {
            let expr = CSTNode::new(
                CSTNodeKind::Statement {
                    statement: Box::new(self.make_factor_and_next()),
                },
                vec![],
            );
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        // Item
        if let Ok(expr) = self.item() {
            self.write_memo(&key, Some(&expr));
            return Ok(CSTNode::new(
                CSTNodeKind::Statement {
                    statement: Box::new(expr),
                },
                vec![],
            ));
        }

        // LetStatement
        if let Ok(expr) = self.let_statement() {
            self.write_memo(&key, Some(&expr));
            return Ok(CSTNode::new(
                CSTNodeKind::Statement {
                    statement: Box::new(expr),
                },
                vec![],
            ));
        }

        // ExpressionStatement
        if let Ok(expr) = self.expression_statement() {
            self.write_memo(&key, Some(&expr));
            return Ok(CSTNode::new(
                CSTNodeKind::Statement {
                    statement: Box::new(expr),
                },
                vec![],
            ));
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // LetStatement ::= OuterAttribute* (`ur` | `sr` | `nr` | `let`)
    //                  PatternNoTopAlt ( `:` Type )?
    //                  (`=` Expression ( `else` BlockExpression)? )? `;`
    fn let_statement(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("LetStatement");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        let mut colon = None;
        let mut type_expression = None;
        let mut equal = None;
        let mut expression = None;
        let mut else_keyword = None;
        let mut block_expression = None;

        // OuterAttribute*
        let mut outer_attribute = Vec::<CSTNode>::new();
        while let Ok(expr) = self.outer_attribute() {
            outer_attribute.push(expr);
        }

        // (`ur` | `sr` | `nr` | `let`)
        let rarity = match self.lexer.peek() {
            Token::Keyword(keyword) => match keyword {
                Keyword::Ur | Keyword::Sr | Keyword::Nr | Keyword::Let => {
                    Box::new(self.make_factor_and_next())
                }
                _ => return self.error(SyntaxError::ExpectedToken, &key),
            },
            _ => return self.error(SyntaxError::ExpectedToken, &key),
        };

        // PatternNoTopAlt
        let pattern_no_top_alt = Box::new(self.pattern_no_top_alt()?);

        // ( `:` Type )?
        if matches!(self.lexer.peek(), Token::Colon) {
            colon = Some(Box::new(self.make_factor_and_next()));

            // Type
            type_expression = Some(Box::new(self.type_expression()?));
        }

        //  (`=` Expression ( `else` BlockExpression)? )? `;`
        if matches!(self.lexer.peek(), Token::Equal) {
            // `=`
            equal = Some(Box::new(self.make_factor_and_next()));

            // Expression
            expression = Some(Box::new(self.expression()?));
            //self.min_bp = 0;

            // `else`
            if let Token::Keyword(keyword) = self.lexer.peek() {
                if !matches!(keyword, Keyword::Else) {
                    return self.error(SyntaxError::ExpectedToken, &key);
                }
                else_keyword = Some(Box::new(self.make_factor_and_next()));

                // BlockExpression
                block_expression = Some(Box::new(self.block_expression()?))
            }
        }

        // ;
        if !matches!(self.lexer.peek(), Token::Semicolon) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let semicolon = Box::new(self.make_factor_and_next());

        let expr = CSTNode::new(
            CSTNodeKind::LetStatement {
                outer_attribute,
                rarity,
                pattern_no_top_alt,
                colon,
                type_expression,
                equal,
                expression,
                else_keyword,
                block_expression,
                semicolon,
            },
            vec![],
        );
        self.write_memo(&key, Some(&expr));

        Ok(expr)
    }

    // ExpressionStatement ::= ExpressionWithoutBlock `;` | ExpressionWithBlock `;`?
    fn expression_statement(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("ExpressionStatement");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // ExpressionWithoutBlock `;`
        if let Ok(mut expr) = self.expression_without_block() {
            if matches!(self.lexer.peek(), Token::Semicolon) {
                expr.children.push(self.make_factor_and_next());
                self.write_memo(&key, Some(&expr));
                return Ok(expr);
            }
        }

        // ExpressionWithBlock `;`?
        if let Ok(expr) = self.expression_with_block() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    //
    // 以下Pattern
    //

    // Pattern ::= `|`? PatternNoTopAlt ( `|` PatternNoTopAlt )*
    fn pattern(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Pattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut or_token = None;

        // `|`?
        if let Token::Or = self.lexer.peek() {
            or_token = Some(Box::new(self.make_factor_and_next()));
        }

        self.pattern_no_top_alt()?;

        while let Ok(expr_pattern_no_top_alt) = self.pattern_no_top_alt() {
            //
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // PatternNoTopAlt ::= PatternWithoutRange | RangePattern
    fn pattern_no_top_alt(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("PatternNoTopAlt");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        if let Ok(expr) = self.pattern_without_range() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // PatternWithoutRange ::= LiteralPattern | IdentifierPattern | WildcardPattern | RestPattern |
    //                         ReferencePattern | StructPattern | TupleStructPattern | TuplePattern | GroupedPattern |
    //                         SlicePattern | PathPattern | MacroInvocation
    fn pattern_without_range(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("PatternWithoutRange");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // LiteralPattern
        if let Ok(expr) = self.literal_pattern() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        // IdentifierPattern
        if let Ok(expr) = self.identifier_pattern() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        // WildcardPattern
        if let Ok(expr) = self.wildcard_pattern() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        // RestPattern
        if let Ok(expr) = self.rest_pattern() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        // ReferencePattern
        if let Ok(expr) = self.reference_pattern() {
            self.write_memo(&key, Some(&expr));
            return Ok(expr);
        }

        self.error(SyntaxError::NotMatch, &key)
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
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        match self.lexer.next() {
            Token::Keyword(keyword) => match keyword {
                Keyword::True => {
                    let node = CSTNode::new(
                        CSTNodeKind::LiteralPattern {
                            literal: Literal::new(LiteralKind::Bool(true), ""),
                        },
                        vec![],
                    );
                    self.write_memo(&key, Some(&node));
                    Ok(node)
                }

                Keyword::False => {
                    let node = CSTNode::new(
                        CSTNodeKind::LiteralPattern {
                            literal: Literal::new(LiteralKind::Bool(false), ""),
                        },
                        vec![],
                    );
                    self.write_memo(&key, Some(&node));
                    Ok(node)
                }
                _ => self.error(SyntaxError::ExpectedToken, &key),
            },
            Token::Literal(literal) => {
                let node = CSTNode::new(CSTNodeKind::LiteralPattern { literal }, vec![]);
                self.write_memo(&key, Some(&node));
                Ok(node)
            }
            //
            Token::Minus => {
                let Token::Literal(literal) = self.lexer.next() else {
                    return self.error(SyntaxError::NotMatch, &key);
                };

                match literal.literal_kind {
                    LiteralKind::Integer | LiteralKind::Float => {
                        let node = CSTNode::new(CSTNodeKind::LiteralPattern { literal }, vec![]);
                        self.write_memo(&key, Some(&node));
                        Ok(node)
                    }
                    _ => self.error(SyntaxError::NotMatch, &key),
                }
            }

            _ => self.error(SyntaxError::NotMatch, &key),
        }
    }

    // IdentifierPattern ::= `ref`? `mut`? Identifier (`@` PatternNoTopAlt )?
    fn identifier_pattern(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("IdentifierPattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        let mut ref_keyword = None;
        let mut mut_keyword = None;
        let mut at_symbol = None;
        let mut pattern_no_top_alt = None;

        // `ref`?
        if let Token::Keyword(keyword) = self.lexer.peek() {
            if matches!(keyword, Keyword::Ref) {
                ref_keyword = Some(Box::new(self.make_factor_and_next()));
            }
        }

        // `mut`?
        if let Token::Keyword(keyword) = self.lexer.peek() {
            if matches!(keyword, Keyword::Mut) {
                mut_keyword = Some(Box::new(self.make_factor_and_next()));
            }
        }

        // Identifier
        let identifier = match self.lexer.peek() {
            Token::Identifier(_) => Box::new(self.make_factor_and_next()),
            _ => return self.error(SyntaxError::NotMatch, &key),
        };

        // (`@` PatternNoTopAlt )?
        if matches!(self.lexer.peek(), Token::At) {
            at_symbol = Some(Box::new(self.make_factor_and_next()));
            pattern_no_top_alt = Some(Box::new(self.pattern_no_top_alt()?));
        }

        let node = CSTNode::new(
            CSTNodeKind::IdentifierPattern {
                ref_keyword,
                mut_keyword,
                identifier,
                at_symbol,
                pattern_no_top_alt,
            },
            vec![],
        );
        self.write_memo(&key, Some(&node));

        Ok(node)
    }

    // WildcardPattern ::= `_`
    fn wildcard_pattern(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("WildcardPattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        match self.lexer.peek() {
            Token::Underscore => {
                let node = CSTNode::new(
                    CSTNodeKind::WildcardPattern {
                        wildcard: Box::new(self.make_factor_and_next()),
                    },
                    vec![],
                );
                self.write_memo(&key, Some(&node));
                Ok(node)
            }
            _ => self.error(SyntaxError::ExpectedToken, &key),
        }
    }

    // RestPattern ::= `..`
    fn rest_pattern(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("RestPattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        match self.lexer.peek_glue() {
            Token::DotDot => {
                let pos = self.lexer.get_sorce_position();
                let node = CSTNode::new(
                    CSTNodeKind::RestPattern {
                        rest: Box::new(CSTNode::new(
                            CSTNodeKind::Factor {
                                token: self.lexer.next_glue(),
                                row: pos.0,
                                column: pos.1,
                            },
                            vec![],
                        )),
                    },
                    vec![],
                );
                self.write_memo(&key, Some(&node));
                Ok(node)
            }
            _ => self.error(SyntaxError::ExpectedToken, &key),
        }
    }

    // ReferencePattern ::= (`&`|`&&`) mut? PatternWithoutRange
    fn reference_pattern(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("ReferencePattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        // (`&`|`&&`)
        if !matches!(self.lexer.peek_glue(), Token::And | Token::AndAnd) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }

        // mut?
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Mut)) {
            return self.error(SyntaxError::ExpectedToken, &key);
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
        CSTNode::new(
            CSTNodeKind::Factor {
                token,
                row: pos.0,
                column: pos.1,
            },
            vec![],
        )
    }

    fn make_factor_and_next(&mut self) -> CSTNode {
        let pos = self.lexer.get_sorce_position();
        let token = self.lexer.next();
        CSTNode::new(
            CSTNodeKind::Factor {
                token,
                row: pos.0,
                column: pos.1,
            },
            vec![],
        )
    }

    fn make_operator_and_next(&mut self) -> CSTNode {
        let pos = self.lexer.get_sorce_position();
        let token = self.lexer.next_glue();
        CSTNode::new(
            CSTNodeKind::Operator {
                token,
                row: pos.0,
                column: pos.1,
            },
            vec![],
        )
    }

    fn make_key(&self, rule: &str) -> ParseMemoKey {
        ParseMemoKey {
            position: self.lexer.get_token_position(),
            rule: rule.to_string(),
        }
    }

    fn write_memo(&mut self, key: &ParseMemoKey, memo: Option<&CSTNode>) {
        self.log.push_str(&format!(
            "WriteMemo {} pos: {:?} token: {:?} \n",
            key.rule,
            key.position,
            self.lexer.peek()
        ));

        if let Some(node) = memo {
            match key.rule.as_str() {
                "Statement" => self.min_bp = 0,
                _ => (),
            };

            self.memo.insert(
                key.clone(),
                Some(ParseMemoValue {
                    node: node.clone(),
                    next_position: self.lexer.get_token_position(),
                }),
            );
        } else {
            self.memo.insert(key.clone(), None);
        }
    }

    fn get_memo(&mut self, key: &ParseMemoKey) -> MemoResult<CSTNode> {
        let Some(node) = self.memo.get(key) else {
            self.log.push_str(&format!(
                "First call to {} pos: {:?} token: {:?} \n",
                key.rule,
                key.position,
                self.lexer.peek()
            ));

            return MemoResult::None;
        };

        let Some(value) = node else {
            self.log.push_str(&format!(
                "Recursed {} pos: {:?} token: {:?}\n",
                key.rule,
                key.position,
                self.lexer.peek()
            ));

            return MemoResult::Recursive;
        };

        self.log.push_str(&format!(
            "Use memo {} pos: {:?} token: {:?}\n",
            key.rule,
            key.position,
            self.lexer.peek()
        ));

        // メモがあった場合解析が進んだ場所まで移動
        self.lexer.set_postion(value.next_position);
        MemoResult::Some(value.node.clone())
    }

    fn backtrack(&mut self, position: usize) {
        self.lexer.set_postion(position);
    }

    // まともなエラー出力用のプロジェクトができるまで仮で
    fn error(&mut self, error_type: SyntaxError, key: &ParseMemoKey) -> Result<CSTNode, Error> {
        self.log.push_str(&format!(
            "Error({:?}) {} pos: {:?} token: {:?}\n",
            error_type,
            key.rule,
            key.position,
            self.lexer.peek()
        ));

        self.backtrack(key.position);

        Err(Error {
            error_kind: ErrorKind::Syntax(error_type),
            error_text: "".to_string(),
        })
    }

    pub fn output_log_file(&self, file_name: &str) {
        let mut log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(file_name)
            .unwrap();
        let Err(_) = write!(log_file, "{}", self.log) else {
            println!("log output error!");
            return;
        };
    }
}
