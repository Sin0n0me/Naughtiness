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
        //println!("{:#?}", token_list);
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // `#`
        if !matches!(self.lexer.peek(), Token::Pound) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        // `!`
        if !matches!(self.lexer.peek(), Token::Not) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        // `[`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Brackets)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        // Attribute
        let attribute = Box::new(self.attribute()?);

        // `]`
        if !matches!(
            self.lexer.peek(),
            Token::RightParenthesis(RightParenthesis::Brackets)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        node.node_kind = CSTNodeKind::InnerAttribute { attribute };
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // `#`
        if !matches!(self.lexer.peek(), Token::Pound) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        // `[`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Brackets)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        let attribute = Box::new(self.attribute()?);

        // `]`
        if !matches!(
            self.lexer.peek(),
            Token::RightParenthesis(RightParenthesis::Brackets)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        node.node_kind = CSTNodeKind::OuterAttribute { attribute };
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

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
            node.node_kind = CSTNodeKind::VisItem {
                visibility,
                item: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // FunctionQualifiers
        let function_qualifiers = Box::new(self.function_qualifiers()?);

        // `fn`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Fn)) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

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
        node.children.push(self.make_factor_and_next());

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
        node.children.push(self.make_factor_and_next());

        // FunctionReturnType?
        let mut function_return_type = None;
        if let Ok(expr) = self.function_return_type() {
            function_return_type = Some(Box::new(expr));
        }

        // WhereClause?
        let mut where_clause = None;

        // ( BlockExpression | `;` )
        if let Token::Semicolon = self.lexer.peek() {
            let semicolon = Box::new(self.make_factor_and_next());
            node.node_kind = CSTNodeKind::Function {
                function_qualifiers,
                identifier,
                generic_params,
                function_parameters,
                function_return_type,
                where_clause,
                block_expression_or_semicolon: semicolon,
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        if let Ok(expr) = self.block_expression() {
            node.node_kind = CSTNodeKind::Function {
                function_qualifiers,
                identifier,
                generic_params,
                function_parameters,
                function_return_type,
                where_clause,
                block_expression_or_semicolon: Box::new(expr),
            };

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

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

        node.node_kind = CSTNodeKind::FunctionQualifiers {
            const_keyword,
            async_keyword,
            item_safety,
            extern_keyword,
            abi,
        };

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

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

    // TODO
    // GenericParams ::= `<` `>` | `<` (GenericParam `,`)* GenericParam `,`? `>`
    fn generic_params(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("GenericParams");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // SelfParam
        let first_self_param = self.self_param();

        // `,`
        let mut comma = if matches!(self.lexer.peek(), Token::Comma) {
            vec![self.make_factor_and_next()]
        } else {
            vec![]
        };
        let has_comma = comma.len() > 0;

        // SelfParamと,両方存在すればFunctionparamの判定へ
        if first_self_param.is_ok() && !has_comma {
            // SelfParamのみ
            node.node_kind = CSTNodeKind::FunctionParametersSelfOnly {
                self_param: Box::new(first_self_param.unwrap()),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        } else if first_self_param.is_err() && has_comma {
            // ,のみはエラー
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let self_param = if first_self_param.is_ok() && has_comma {
            Some(Box::new(first_self_param.unwrap()))
        } else {
            None
        };

        // FunctionParam
        let mut function_param = vec![self.function_param()?];

        // (`,` FunctionParam)* `,`?
        while let Token::Comma = self.lexer.peek() {
            // `,`
            comma.push(self.make_factor_and_next());

            // FunctionParam
            let Ok(param) = self.function_param() else {
                break;
            };

            function_param.push(param);
        }

        // `,`
        if matches!(self.lexer.peek(), Token::Comma) {
            comma.push(self.make_factor_and_next())
        }

        node.node_kind = CSTNodeKind::FunctionParameters {
            self_param,
            function_param,
        };

        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // SelfParam ::= OuterAttribute* ( ShorthandSelf | TypedSelf )
    fn self_param(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("SelfParam");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // OuterAttribute*
        let mut outer_attribute = Vec::<CSTNode>::new();
        while let Ok(expr) = self.outer_attribute() {
            outer_attribute.push(expr);
        }

        // ( ShorthandSelf | TypedSelf )
        if let Ok(expr) = self.shorthand_self() {
            node.node_kind = CSTNodeKind::SelfParam {
                outer_attribute,
                self_kind: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        } else if let Ok(expr) = self.typed_self() {
            node.node_kind = CSTNodeKind::SelfParam {
                outer_attribute,
                self_kind: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // `mut`?
        let mut mut_keyword = None;
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Mut)) {
            mut_keyword = Some(Box::new(self.make_factor_and_next()));
        }

        // `self`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::SelfValue)) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        // `:`
        if !matches!(self.lexer.peek(), Token::Colon) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        // Type
        let type_expr = Box::new(self.type_expression()?);

        node.node_kind = CSTNodeKind::TypedSelf {
            mut_keyword,
            type_expr,
        };

        self.write_memo(&key, Some(&node));
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // OuterAttribute*
        let mut outer_attribute = Vec::<CSTNode>::new();
        while let Ok(expr) = self.outer_attribute() {
            outer_attribute.push(expr);
        }

        // `...`
        if let Token::DotDotDot = self.lexer.peek() {
            let pattern = Box::new(self.make_factor_and_next());
            node.node_kind = CSTNodeKind::FunctionParam {
                outer_attribute,
                pattern,
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // FunctionParamPattern
        if let Ok(expr) = self.function_param_pattern() {
            let pattern = Box::new(expr);
            node.node_kind = CSTNodeKind::FunctionParam {
                outer_attribute,
                pattern,
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // Type
        if let Ok(expr) = self.type_expression() {
            let pattern = Box::new(expr);
            node.node_kind = CSTNodeKind::FunctionParam {
                outer_attribute,
                pattern,
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // PatternNoTopAlt
        let pattern_no_top_alt = Box::new(self.pattern_no_top_alt()?);

        // `:`
        if !matches!(self.lexer.peek(), Token::Colon) {
            return self.error(SyntaxError::NotMatch, &key);
        }

        // ( Type | `...` )
        if let Ok(expr) = self.type_expression() {
            let pattern = Box::new(expr);
            node.node_kind = CSTNodeKind::FunctionParamPattern {
                pattern_no_top_alt,
                pattern,
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }
        if matches!(self.lexer.peek_glue(), Token::DotDotDot) {
            let pattern = Box::new(self.make_factor_and_next_glue());
            node.node_kind = CSTNodeKind::FunctionParamPattern {
                pattern_no_top_alt,
                pattern,
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // TODO
        self.error(SyntaxError::NotMatch, &key)
    }

    // Structs

    //
    // Type
    //

    // Type ::= TypeNoBounds | ImplTraitType | TraitObjectType
    fn type_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Type");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        if let Ok(expr) = self.type_no_bounds() {
            node.node_kind = CSTNodeKind::Type {
                type_pattern: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // TypeNoBounds ::= ParenthesizedType | ImplTraitTypeOneBound | TraitObjectTypeOneBound | TypePath | TupleType
    //                | NeverType | RawPointerType | ReferenceType | ArrayType | SliceType | InferredType
    //                | QualifiedPathInType | BareFunctionType | MacroInvocation
    fn type_no_bounds(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("TypeNoBounds");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // ParenthesizedType
        if let Ok(expr) = self.parenthesized_type() {
            node.node_kind = CSTNodeKind::TypeNoBounds {
                type_pattern: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // ParenthesizedType ::= `(` Type `)`
    fn parenthesized_type(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("ParenthesizedType");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // `(`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        // Type
        let type_expression = Box::new(self.type_expression()?);

        // `)`
        if !matches!(
            self.lexer.peek(),
            Token::RightParenthesis(RightParenthesis::Parenthesis)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        node.node_kind = CSTNodeKind::ParenthesizedType { type_expression };

        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // TypePath ::= `::`? TypePathSegment (`::` TypePathSegment)*
    fn type_path(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("TypePath");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // `::`?
        if matches!(self.lexer.peek_glue(), Token::PathSeparater) {
            node.children.push(self.make_factor_and_next());
        }

        // TypePathSegment
        let mut type_path_segment = Vec::new();
        type_path_segment.push(self.type_path_segment()?);

        // (`::` TypePathSegment)*
        while let Ok(expr) = self.type_path_segment() {
            type_path_segment.push(expr);
        }

        node.node_kind = CSTNodeKind::TypePath { type_path_segment };

        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // TypePathSegment ::= PathIdentSegment (`::`? (GenericArgs | TypePathFn))?
    fn type_path_segment(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("TypePathSegment");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // PathIdentSegment
        let path_ident_segment = self.path_ident_segment()?;

        // `::`?
        if matches!(self.lexer.peek_glue(), Token::PathSeparater) {}

        // TODO GenericArgs
        if let Ok(expr) = self.type_path_fn() {
            //
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // TODO
    // TypePathFn ::= `(` TypePathFnInputs? `)` (`->` TypeNoBounds)?
    fn type_path_fn(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("TypePathFn");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // `(`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        // TypePathFnInputs?
        let mut type_path_fn_inputs = None;
        if let Ok(expr) = self.type_path_fn_inputs() {
            type_path_fn_inputs = Some(Box::new(expr));
        }

        // `)`
        if !matches!(
            self.lexer.peek(),
            Token::RightParenthesis(RightParenthesis::Parenthesis)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        let right_parenthesis = self.make_factor_and_next();

        // (`->` TypeNoBounds)?
        if matches!(self.lexer.peek_glue(), Token::RightAllow) {
            let right_allow = Box::new(self.lexer.next_glue());
            let type_no_bounds = Box::new(self.type_no_bounds()?);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // TypePathFnInputs ::= Type (`,` Type)* `,`?
    fn type_path_fn_inputs(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("TypePathFnInputs");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        let mut type_expr = vec![self.type_expression()?];
        while let Token::Comma = self.lexer.peek() {
            node.children.push(self.make_factor_and_next());

            let Ok(expr) = self.type_expression() else {
                break;
            };

            type_expr.push(expr);
        }

        if matches!(self.lexer.peek(), Token::Comma) {
            node.children.push(self.make_factor_and_next());
        }

        node.node_kind = CSTNodeKind::TypePathFnInputs { type_expr };

        self.write_memo(&key, Some(&node));

        Ok(node)
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        if let Ok(expr) = self.expression_without_block() {
            node.node_kind = CSTNodeKind::Expression {
                expression: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        if let Ok(expr) = self.expression_with_block() {
            node.node_kind = CSTNodeKind::Expression {
                expression: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // OuterAttribute*
        let mut outer_attribute = Vec::<CSTNode>::new();
        while let Ok(expr) = self.outer_attribute() {
            outer_attribute.push(expr);
        }

        // OperatorExpression
        if let Ok(expr) = self.operator_expression() {
            node.node_kind = CSTNodeKind::ExpressionWithoutBlock {
                outer_attribute,
                expression: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // LiteralExpression
        if let Ok(expr) = self.literal_expression() {
            node.node_kind = CSTNodeKind::ExpressionWithoutBlock {
                outer_attribute,
                expression: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // PathExpression
        if let Ok(expr) = self.path_expression() {
            node.node_kind = CSTNodeKind::ExpressionWithoutBlock {
                outer_attribute,
                expression: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // GroupedExpression
        if let Ok(expr) = self.grouped_expression() {
            node.node_kind = CSTNodeKind::ExpressionWithoutBlock {
                outer_attribute,
                expression: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // StructExpression
        if let Ok(expr) = self.struct_expression() {
            node.node_kind = CSTNodeKind::ExpressionWithoutBlock {
                outer_attribute,
                expression: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // CallExpression
        if let Ok(expr) = self.call_expression() {
            node.node_kind = CSTNodeKind::ExpressionWithoutBlock {
                outer_attribute,
                expression: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // MethodCallExpression
        if let Ok(expr) = self.method_call_expression() {
            node.node_kind = CSTNodeKind::ExpressionWithoutBlock {
                outer_attribute,
                expression: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // ReturnExpression
        if let Ok(expr) = self.return_expression() {
            node.node_kind = CSTNodeKind::ExpressionWithoutBlock {
                outer_attribute,
                expression: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        let index = self.lexer.get_token_position();
        node.node_kind = match self.lexer.peek() {
            Token::Literal(literal) => CSTNodeKind::Literal { literal, index },
            Token::Keyword(Keyword::True) => CSTNodeKind::Literal {
                literal: Literal::new(LiteralKind::Bool(true), ""),
                index,
            },
            Token::Keyword(Keyword::False) => CSTNodeKind::Literal {
                literal: Literal::new(LiteralKind::Bool(false), ""),
                index,
            },

            _ => return self.error(SyntaxError::ExpectedToken, &key),
        };

        self.lexer.next();

        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // PathExpression ::= PathInExpression | QualifiedPathInExpression
    fn path_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("PathExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // PathInExpression
        if let Ok(expr) = self.path_in_expression() {
            node.node_kind = CSTNodeKind::PathExpression {
                path_in_expression: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // QualifiedPathInExpression
        if let Ok(expr) = self.qualified_path_in_expression() {
            node.node_kind = CSTNodeKind::PathExpression {
                path_in_expression: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        //  `::`?
        if matches!(self.lexer.peek_glue(), Token::PathSeparater) {
            node.children.push(self.make_factor_and_next_glue());
        }

        // PathExprSegment
        let mut path_expr_segment = vec![self.path_expr_segment()?];

        // (`::` PathExprSegment)*
        while let Token::PathSeparater = self.lexer.peek_glue() {
            // `::`
            node.children.push(self.make_factor_and_next_glue());

            // PathExprSegment
            let Ok(expr) = self.path_expr_segment() else {
                break;
            };

            path_expr_segment.push(expr);
        }

        node.node_kind = CSTNodeKind::PathInExpression { path_expr_segment };
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // PathIdentSegment
        let path_ident_segment = Box::new(self.path_ident_segment()?);

        //  (`::` GenericArgs)?
        let mut generic_args = None;
        if matches!(self.lexer.peek_glue(), Token::PathSeparater) {
            node.children.push(self.make_factor_and_next());
            generic_args = Some(Box::new(self.generic_args()?));
        }

        node.node_kind = CSTNodeKind::PathExprSegment {
            path_ident_segment,
            generic_args,
        };

        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // PathIdentSegment   ::= Identifier | `super` | `self` | `Self` | `crate` | `$crate`
    fn path_ident_segment(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("PathIdentSegment");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);
        match self.lexer.peek() {
            Token::Identifier(_) => {
                node.node_kind = CSTNodeKind::PathIdentSegment {
                    path_ident_segment: Box::new(self.make_factor_and_next()),
                };

                self.write_memo(&key, Some(&node));
                Ok(node)
            }
            Token::Keyword(keyword) => match keyword {
                Keyword::Super | Keyword::SelfValue | Keyword::SelfType | Keyword::Crate => {
                    node.node_kind = CSTNodeKind::PathIdentSegment {
                        path_ident_segment: Box::new(self.make_factor_and_next()),
                    };

                    self.write_memo(&key, Some(&node));
                    Ok(node)
                }
                _ => self.error(SyntaxError::NotMatch, &key),
            },

            _ => self.error(SyntaxError::NotMatch, &key),
        }
    }

    // TODO
    // GenericArgs ::= `<` `>` | `<` ( GenericArg `,` )* GenericArg `,`? `>`
    fn generic_args(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("GenericArgs");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // `<`
        if !matches!(self.lexer.peek(), Token::LessThan) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        // `>`
        if !matches!(self.lexer.peek(), Token::GreaterThan) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        self.error(SyntaxError::NotMatch, &key)
    }

    // TODO
    // GenericArg ::= Lifetime | Type | GenericArgsConst | GenericArgsBinding | GenericArgsBounds
    fn generic_arg(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("GenericArg");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        //if let Ok(expr) = self.life_time() {}

        // Type
        if let Ok(expr) = self.type_expression() {
            return Ok(expr);
        }

        // GenericArgsConst
        if let Ok(expr) = self.generic_args_const() {
            return Ok(expr);
        }

        // GenericArgsBinding
        if let Ok(expr) = self.generic_args_bounds() {
            return Ok(expr);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // GenericArgsConst ::= BlockExpression | LiteralExpression | `-` LiteralExpression | SimplePathSegment
    fn generic_args_const(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("GenericArgsConst");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // BlockExpression
        if let Ok(expr) = self.block_expression() {
            node.node_kind = CSTNodeKind::GenericArgsConst {
                expression: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // LiteralExpression
        if let Ok(expr) = self.literal_expression() {
            node.node_kind = CSTNodeKind::GenericArgsConst {
                expression: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // `-` LiteralExpression
        if matches!(self.lexer.peek(), Token::Minus) {
            node.children.push(self.make_factor_and_next());

            if let Ok(expr) = self.literal_expression() {
                node.node_kind = CSTNodeKind::GenericArgsConst {
                    expression: Box::new(expr),
                };

                self.write_memo(&key, Some(&node));
                return Ok(node);
            }
        }

        // SimplePathSegment
        if let Ok(expr) = self.simple_path_segment() {
            node.node_kind = CSTNodeKind::GenericArgsConst {
                expression: Box::new(expr),
            };

            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // TODO
    // GenericArgsBinding ::= Identifier GenericArgs? `=` Type
    fn generic_arg_binding(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("GenericArgsBinding");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // Identifier
        let ident = match self.lexer.peek() {
            Token::Identifier(ident) => {}
            _ => return self.error(SyntaxError::ExpectedToken, &key),
        };

        if let Ok(expr) = self.generic_args() {
            // TODO
        }

        // `=`
        if !matches!(self.lexer.peek(), Token::Equal) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }

        let type_expr = self.type_expression()?;

        self.error(SyntaxError::NotMatch, &key)
    }

    // TODO
    // GenericArgsBounds ::= Identifier GenericArgs? `:` TypeParamBounds
    fn generic_args_bounds(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("GenericArgsBounds");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // Identifier
        let ident = match self.lexer.peek() {
            Token::Identifier(ident) => {}
            _ => return self.error(SyntaxError::ExpectedToken, &key),
        };

        // GenericArg?
        let mut generic_arg = None;
        if let Ok(expr) = self.generic_args() {
            generic_arg = Some(Box::new(expr));
        }

        // `:`
        if !matches!(self.lexer.peek(), Token::Colon) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }

        // TypeParamBounds
        // self.type_param_bounds()?;

        self.error(SyntaxError::NotMatch, &key)
    }

    // QualifiedPathInExpression ::= QualifiedPathType (`::` PathExprSegment)+
    fn qualified_path_in_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("QualifiedPathInExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // QualifiedPathType
        let qualified_path_type = Box::new(self.qualified_path_type()?);

        // (`::` PathExprSegment)+
        let mut path_expr_segment = vec![];
        while let Token::PathSeparater = self.lexer.peek_glue() {
            node.children.push(self.make_factor_and_next_glue());

            let Ok(expr) = self.path_expr_segment() else {
                break;
            };

            path_expr_segment.push(expr);
        }

        if path_expr_segment.len() == 0 {
            return self.error(SyntaxError::NotMatch, &key);
        }

        node.node_kind = CSTNodeKind::QualifiedPathInExpression {
            qualified_path_type,
            path_expr_segment,
        };

        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // QualifiedPathType ::= `<` Type (`as` TypePath)? `>`
    fn qualified_path_type(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("QualifiedPathType");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        self.type_expression()?;

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // QualifiedPathType
        self.qualified_path_type()?;

        // (`::` TypePathSegment)+
        self.type_path_segment();

        self.error(SyntaxError::NotMatch, &key)
    }

    // SimplePath ::= `::`? SimplePathSegment (`::` SimplePathSegment)*
    fn simple_path(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("SimplePath");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        self.simple_path_segment()?;

        self.error(SyntaxError::NotMatch, &key)
    }

    // SimplePathSegment ::= Identifier | `super` | `self` | `crate` | `$crate`
    fn simple_path_segment(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("SimplePathSegment");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);
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
            let token = self.lexer.next_glue();
            let rhs = Some(Box::new(self.operator_expression()?));
            self.make_operator(token, rhs, None)
        } else {
            // TODO Expressionの最初に呼び出す
            // Expressionの再帰用に呼び出し元だけを削除
            self.memo.remove(&self.make_key("Expression"));
            self.memo.remove(&self.make_key("ExpressionWithoutBlock"));
            self.expression()?
        };

        loop {
            let op = self.lexer.peek_glue();
            if !is_operator(&op) {
                break;
            }

            // 後置演算子
            if let Some((left_bp, ())) = postfix_binding_power(&op) {
                if left_bp < min_bp {
                    break;
                }

                let token = self.lexer.next_glue();
                lhs = self.make_operator(token, Some(Box::new(lhs)), None);
                continue;
            }

            // 中置演算子
            if let Some((left_bp, right_bp)) = infix_binding_power(&op) {
                if left_bp < min_bp {
                    break;
                }
                self.min_bp = right_bp; // 次の再帰のために保存
                let token = self.lexer.next_glue();
                let rhs = self.operator_expression()?;

                lhs = self.make_operator(token, Some(Box::new(lhs)), Some(Box::new(rhs)));
                continue;
            }

            break;
        }

        self.write_memo(&key, Some(&lhs));
        Ok(lhs)
    }

    // TODO
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

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

    // TODO
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // TODO

        // `(`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        // Expression
        let expression = Box::new(self.expression()?);

        // `)`
        if !matches!(
            self.lexer.peek(),
            Token::RightParenthesis(RightParenthesis::Parenthesis)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        node.node_kind = CSTNodeKind::GroupedExpression { expression };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // StructExpression ::= StructExprStruct | StructExprTuple | StructExprUnit
    fn struct_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("StructExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // StructExprStruct
        if let Ok(expr) = self.struct_expr_struct() {
            node.node_kind = CSTNodeKind::StructExpression {
                expression: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // StructExprTuple
        if let Ok(expr) = self.struct_expr_tuple() {
            node.node_kind = CSTNodeKind::StructExpression {
                expression: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // StructExprUnit
        if let Ok(expr) = self.struct_expr_unit() {
            node.node_kind = CSTNodeKind::StructExpression {
                expression: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        let path_in_expression = Box::new(self.path_in_expression()?);

        // `{`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Brace)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

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
        node.children.push(self.make_factor_and_next());

        node.node_kind = CSTNodeKind::StructExprStruct {
            path_in_expression,
            expression,
        };
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // StructExprField
        let mut struct_expr_filed = vec![self.struct_expr_filed()?];

        // (, StructExprField)*
        while let Token::Comma = self.lexer.peek() {
            // `,`
            node.children.push(self.make_factor_and_next());

            let Ok(expr) = self.struct_expr_filed() else {
                break;
            };

            struct_expr_filed.push(expr);
        }

        // (, StructBase | ,?)
        let mut struct_base = None;
        if matches!(self.lexer.peek(), Token::Comma) {
            node.children.push(self.make_factor_and_next());
            if let Ok(expr) = self.struct_base() {
                struct_base = Some(Box::new(expr));
            }
        }

        node.node_kind = CSTNodeKind::StructExprFields {
            struct_expr_filed,
            struct_base,
        };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // StructExprField  ::= OuterAttribute* ( Identifier | (Identifier | tuple_index ) `:` Expression )
    fn struct_expr_filed(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("StructExprField");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

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
                self.make_factor_and_next()
            }
            Token::Literal(literal) => match literal.literal_kind {
                LiteralKind::Integer => self.make_factor_and_next(),
                _ => return self.error(SyntaxError::NotMatch, &key),
            },
            _ => return self.error(SyntaxError::ExpectedToken, &key),
        };

        // `:`
        if !matches!(self.lexer.peek(), Token::Colon) {
            if !is_identifier {
                return self.error(SyntaxError::NotMatch, &key);
            }

            node.node_kind = CSTNodeKind::StructExprField1 {
                outer_attribute,
                identifier: Box::new(identifier_or_tuple),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }
        node.children.push(self.make_factor_and_next());

        // Expression
        let expression = Box::new(self.expression()?);

        node.node_kind = CSTNodeKind::StructExprField2 {
            outer_attribute,
            identifier_or_tuple: Box::new(identifier_or_tuple),
            expression,
        };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // StructBase ::= `..` Expression
    fn struct_base(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("StructBase");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // `..`
        if !matches!(self.lexer.peek(), Token::DotDot) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        // Expression
        let expression = Box::new(self.expression()?);

        node.node_kind = CSTNodeKind::StructBase { expression };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // StructExprTuple ::=  PathInExpression `(` ( Expression (, Expression)* ,? )? `)`
    fn struct_expr_tuple(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("StructExprTuple");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // PathInExpression
        let path_in_expression = self.path_in_expression()?;

        // TODO

        self.error(SyntaxError::ExpectedToken, &key)
    }

    // StructExprUnit ::= PathInExpression
    fn struct_expr_unit(&mut self) -> Result<CSTNode, Error> {
        self.path_in_expression()
    }

    // CallExpression ::= Expression `(` CallParams? `)`
    fn call_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("CallExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // Expression
        let expression = Box::new(self.expression()?);

        // `(`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

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
        node.children.push(self.make_factor_and_next());

        node.node_kind = CSTNodeKind::CallExpression {
            expression,
            call_params,
        };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // CallParams ::= Expression ( `,` Expression )* `,`?
    fn call_params(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("CallParams");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // Expression
        let mut expression = vec![self.expression()?];

        // ( `,` Expression )*
        while let Token::Comma = self.lexer.peek() {
            node.children.push(self.make_factor_and_next());

            let Ok(expr) = self.expression() else {
                break;
            };

            expression.push(expr);
        }

        if matches!(self.lexer.peek(), Token::Comma) {
            node.children.push(self.make_factor());
        }

        node.node_kind = CSTNodeKind::CallParams { expression };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // TODO
    // MethodCallExpression ::= Expression `.` PathExprSegment `(` CallParams? `)`
    fn method_call_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("MethodCallExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // Expression
        self.expression()?;

        // `.`
        if !matches!(self.lexer.peek(), Token::Dot) {
            return self.error(SyntaxError::NotMatch, &key);
        }

        // PathExprSegment
        let path_expr_segment = self.path_expr_segment()?;

        // `(`

        // CallParams?
        let mut call_params = None;
        if let Ok(expr) = self.call_params() {
            call_params = Some(Box::new(expr));
        }

        // `)`

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);
        // `return`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Return)) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        node.children.push(self.make_factor_and_next());

        // Expression?
        let mut expression = None;
        if let Ok(expr) = self.expression() {
            expression = Some(Box::new(expr));
        }

        node.node_kind = CSTNodeKind::ReturnExpression { expression };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // TODO
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);
        // OuterAttribute*
        let mut outer_attribute = Vec::<CSTNode>::new();
        while let Ok(expr) = self.outer_attribute() {
            outer_attribute.push(expr);
        }

        // BlockExpression
        if let Ok(expr) = self.block_expression() {
            node.node_kind = CSTNodeKind::ExpressionWithBlock {
                outer_attribute,
                expression_with_block: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // IfExpression
        if let Ok(expr) = self.if_expression() {
            node.node_kind = CSTNodeKind::ExpressionWithBlock {
                outer_attribute,
                expression_with_block: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // IfLetExpression
        if let Ok(expr) = self.if_let_expression() {
            node.node_kind = CSTNodeKind::ExpressionWithBlock {
                outer_attribute,
                expression_with_block: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // LoopExpression
        if let Ok(expr) = self.loop_expression() {
            node.node_kind = CSTNodeKind::ExpressionWithBlock {
                outer_attribute,
                expression_with_block: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // `{`
        if !matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Brace)
        ) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

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
        node.children.push(self.make_factor_and_next());

        node.node_kind = CSTNodeKind::BlockExpression {
            inner_attribute: Vec::new(),
            statements,
        };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // Statements ::= Statement+ | Statement+ ExpressionWithoutBlock | ExpressionWithoutBlock
    fn statements(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("Statements");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);
        let mut statements = vec![];

        // Statement+ | Statement+ ExpressionWithoutBlock
        if let Ok(expr1) = self.statement() {
            statements.push(expr1);

            while let Ok(expr2) = self.statement() {
                statements.push(expr2);
            }

            // ExpressionWithoutBlock
            if let Ok(expr3) = self.expression_without_block() {
                statements.push(expr3);
            }

            node.node_kind = CSTNodeKind::Statements { statements };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // ExpressionWithoutBlock
        if let Ok(expr) = self.expression_without_block() {
            statements.push(expr);

            node.node_kind = CSTNodeKind::Statements { statements };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // TODO
    // ConstBlockExpression ::= `const` BlockExpression
    fn const_block_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("ConstBlockExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // `const`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Const)) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        let const_keyword = self.make_factor_and_next();

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);
        // `if`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::If)) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        node.children.push(self.make_factor_and_next());

        // Expression
        let expression = Box::new(self.expression()?);

        // BlockExpression
        let block_expression = Box::new(self.block_expression()?);

        // ( `else` ( BlockExpression | IfExpression | IfLetExpression ) )?
        let mut else_expression = None;
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Else)) {
            node.children.push(self.make_factor_and_next());

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

        node.node_kind = CSTNodeKind::IfExpression {
            expression,
            block_expression,
            else_expression,
        };
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // `if`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::If)) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        node.children.push(self.make_factor_and_next());

        // `let`
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Let)) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        node.children.push(self.make_factor_and_next());

        // Pattern
        let pattern = Box::new(self.pattern()?);

        // =
        if !matches!(self.lexer.peek(), Token::Equal) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        node.children.push(self.make_factor_and_next());

        // Scrutinee
        // ** except lazy boolean operator expression **
        let scrutinee = Box::new(self.scrutinee()?);

        // BlockExpression
        let block_expression = Box::new(self.block_expression()?);

        // ( else ( BlockExpression | IfExpression | IfLetExpression ) )?
        let mut else_expression = None;
        if matches!(self.lexer.peek(), Token::Keyword(Keyword::Else)) {
            node.children.push(self.make_factor_and_next());

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

        node.node_kind = CSTNodeKind::IfLetExpression {
            pattern,
            scrutinee,
            block_expression,
            else_expression,
        };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // TODO
    // MatchExpression ::= `match` Scrutinee `{` InnerAttribute* MatchArms? `}`
    fn match_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("MatchExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);
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
        let mut inner_attribute = vec![];
        while let Ok(expr) = self.inner_attribute() {
            inner_attribute.push(expr);
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // ;
        if matches!(self.lexer.peek(), Token::Semicolon) {
            node.node_kind = CSTNodeKind::Statement {
                statement: Box::new(self.make_factor_and_next()),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // Item
        if let Ok(expr) = self.item() {
            node.node_kind = CSTNodeKind::Statement {
                statement: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // LetStatement
        if let Ok(expr) = self.let_statement() {
            node.node_kind = CSTNodeKind::Statement {
                statement: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // ExpressionStatement
        if let Ok(expr) = self.expression_statement() {
            node.node_kind = CSTNodeKind::Statement {
                statement: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        let mut type_expression = None;
        let mut expression = None;
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
            node.children.push(self.make_factor_and_next());

            // Type
            type_expression = Some(Box::new(self.type_expression()?));
        }

        //  (`=` Expression ( `else` BlockExpression)? )? `;`
        if matches!(self.lexer.peek(), Token::Equal) {
            // `=`
            node.children.push(self.make_factor_and_next());

            // Expression
            expression = Some(Box::new(self.expression()?));

            // `else`
            if let Token::Keyword(keyword) = self.lexer.peek() {
                if !matches!(keyword, Keyword::Else) {
                    return self.error(SyntaxError::ExpectedToken, &key);
                }
                node.children.push(self.make_factor_and_next());

                // BlockExpression
                block_expression = Some(Box::new(self.block_expression()?))
            }
        }

        // ;
        if !matches!(self.lexer.peek(), Token::Semicolon) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }
        node.children.push(self.make_factor_and_next());

        node.node_kind = CSTNodeKind::LetStatement {
            outer_attribute,
            rarity,
            pattern_no_top_alt,
            type_expression,
            expression,
            block_expression,
        };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // ExpressionStatement ::= ExpressionWithoutBlock `;` | ExpressionWithBlock `;`?
    fn expression_statement(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("ExpressionStatement");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // ExpressionWithoutBlock `;`
        if let Ok(expr) = self.expression_without_block() {
            if matches!(self.lexer.peek(), Token::Semicolon) {
                node.children.push(self.make_factor_and_next());

                node.node_kind = CSTNodeKind::ExpressionStatement {
                    expression: Box::new(expr),
                };
                self.write_memo(&key, Some(&node));
                return Ok(node);
            }
        }

        // ExpressionWithBlock `;`?
        if let Ok(expr) = self.expression_with_block() {
            // `;`?
            if matches!(self.lexer.peek(), Token::Semicolon) {
                node.children.push(self.make_factor_and_next());
            }

            node.node_kind = CSTNodeKind::ExpressionStatement {
                expression: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // `|`?
        if let Token::Or = self.lexer.peek() {
            node.children.push(self.make_factor_and_next());
        }

        // PatternNoTopAlt
        let mut pattern = vec![self.pattern_no_top_alt()?];

        //  ( `|` PatternNoTopAlt )*
        while let Token::Or = self.lexer.peek() {
            // `|`
            node.children.push(self.make_factor_and_next());

            // PatternNoTopAlt
            let Ok(expr) = self.pattern_no_top_alt() else {
                break;
            };

            pattern.push(expr);
        }

        node.node_kind = CSTNodeKind::Pattern { pattern };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // PatternNoTopAlt ::= PatternWithoutRange | RangePattern
    fn pattern_no_top_alt(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("PatternNoTopAlt");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // PatternWithoutRange
        if let Ok(expr) = self.pattern_without_range() {
            node.node_kind = CSTNodeKind::PatternNoTopAlt {
                pattern: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // RangePattern

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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // LiteralPattern
        if let Ok(expr) = self.literal_pattern() {
            node.node_kind = CSTNodeKind::PatternWithoutRange {
                pattern: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // IdentifierPattern
        if let Ok(expr) = self.identifier_pattern() {
            node.node_kind = CSTNodeKind::PatternWithoutRange {
                pattern: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // WildcardPattern
        if let Ok(expr) = self.wildcard_pattern() {
            node.node_kind = CSTNodeKind::PatternWithoutRange {
                pattern: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // RestPattern
        if let Ok(expr) = self.rest_pattern() {
            node.node_kind = CSTNodeKind::PatternWithoutRange {
                pattern: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // ReferencePattern
        if let Ok(expr) = self.reference_pattern() {
            node.node_kind = CSTNodeKind::PatternWithoutRange {
                pattern: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        let node_kind = match self.lexer.peek() {
            Token::Keyword(Keyword::True) => CSTNodeKind::LiteralPattern {
                literal: Literal::new(LiteralKind::Bool(true), "true"),
            },
            Token::Keyword(Keyword::False) => CSTNodeKind::LiteralPattern {
                literal: Literal::new(LiteralKind::Bool(false), "false"),
            },
            Token::Literal(literal) => CSTNodeKind::LiteralPattern { literal },
            Token::Minus => {
                //  `-`? INTEGER_LITERAL | `-`? FLOAT_LITERAL
                self.lexer.next();
                let Token::Literal(literal) = self.lexer.peek() else {
                    return self.error(SyntaxError::NotMatch, &key);
                };

                if !matches!(
                    literal.literal_kind,
                    LiteralKind::Integer | LiteralKind::Float
                ) {
                    return self.error(SyntaxError::NotMatch, &key);
                }
                CSTNodeKind::LiteralPattern { literal }
            }

            _ => return self.error(SyntaxError::NotMatch, &key),
        };

        node.node_kind = node_kind;
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // IdentifierPattern ::= `ref`? `mut`? Identifier (`@` PatternNoTopAlt )?
    fn identifier_pattern(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("IdentifierPattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        let mut ref_keyword = None;
        let mut mut_keyword = None;
        let mut pattern_no_top_alt = None;

        // `ref`?
        if let Token::Keyword(Keyword::Ref) = self.lexer.peek() {
            ref_keyword = Some(Box::new(self.make_factor_and_next()));
        }

        // `mut`?
        if let Token::Keyword(Keyword::Mut) = self.lexer.peek() {
            mut_keyword = Some(Box::new(self.make_factor_and_next()));
        }

        // Identifier
        let identifier = match self.lexer.peek() {
            Token::Identifier(_) => Box::new(self.make_factor_and_next()),
            _ => return self.error(SyntaxError::NotMatch, &key),
        };

        // (`@` PatternNoTopAlt )?
        if matches!(self.lexer.peek(), Token::At) {
            node.children.push(self.make_factor_and_next());
            pattern_no_top_alt = Some(Box::new(self.pattern_no_top_alt()?));
        }

        node.node_kind = CSTNodeKind::IdentifierPattern {
            ref_keyword,
            mut_keyword,
            identifier,
            pattern_no_top_alt,
        };
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
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        let Token::Underscore = self.lexer.next() else {
            return self.error(SyntaxError::ExpectedToken, &key);
        };
        let wildcard = Box::new(self.make_factor_and_next());

        node.node_kind = CSTNodeKind::WildcardPattern { wildcard };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // RestPattern ::= `..`
    fn rest_pattern(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("RestPattern");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        let Token::DotDot = self.lexer.next_glue() else {
            return self.error(SyntaxError::ExpectedToken, &key);
        };
        let rest = Box::new(self.make_factor_and_next_glue());

        node.node_kind = CSTNodeKind::RestPattern { rest };
        self.write_memo(&key, Some(&node));
        Ok(node)
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
    // LoopExpression
    //

    // LoopExpression ::= LoopLabel?
    //                  (
    //                    InfiniteLoopExpression
    //                  | PredicateLoopExpression
    //                  | PredicatePatternLoopExpression
    //                  | IteratorLoopExpression
    //                  | LabelBlockExpression
    //                  )
    fn loop_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("LoopExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);
        let loop_label = None;

        // InfiniteLoopExpression
        if let Ok(expr) = self.infinite_loop_expression() {
            node.node_kind = CSTNodeKind::LoopExpression {
                loop_label,
                loop_expression: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        // PredicatePatternLoopExpression
        if let Ok(expr) = self.predicate_loop_expression() {
            node.node_kind = CSTNodeKind::LoopExpression {
                loop_label,
                loop_expression: Box::new(expr),
            };
            self.write_memo(&key, Some(&node));
            return Ok(node);
        }

        self.error(SyntaxError::NotMatch, &key)
    }

    // InfiniteLoopExpression ::= `loop` BlockExpression
    fn infinite_loop_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("InfiniteLoopExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);
        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::Loop)) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        node.children.push(self.make_factor_and_next());

        let block_expression = Box::new(self.block_expression()?);
        node.node_kind = CSTNodeKind::InfiniteLoopExpression { block_expression };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    // PredicateLoopExpression ::= `while` Expression BlockExpression
    fn predicate_loop_expression(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("PredicateLoopExpression");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        if !matches!(self.lexer.peek(), Token::Keyword(Keyword::While)) {
            return self.error(SyntaxError::NotMatch, &key);
        }
        node.children.push(self.make_factor_and_next());

        // Expression
        let expression = Box::new(self.expression()?);

        // BlockExpression
        let block_expression = Box::new(self.block_expression()?);

        node.node_kind = CSTNodeKind::PredicateLoopExpression {
            expression,
            block_expression,
        };
        self.write_memo(&key, Some(&node));
        Ok(node)
    }

    //
    // Macro
    //

    // TODO
    // MacroInvocation ::= SimplePath `!` DelimTokenTree
    fn macro_invocation(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("MacroInvocation");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // SimplePath
        let simple_path = self.simple_path()?;

        // `!`
        if !matches!(self.lexer.peek(), Token::Not) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }

        // DelimTokenTree
        let delim_token_tree = self.delim_token_tree()?;

        self.error(SyntaxError::ExpectedToken, &key)
    }

    // DelimTokenTree ::= `(` TokenTree* `)` | `[` TokenTree* `]` | `{` TokenTree* `}`
    fn delim_token_tree(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("DelimTokenTree");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };

        self.error(SyntaxError::ExpectedToken, &key)
    }

    // TokenTree ::= Token | DelimTokenTree
    fn token_tree(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("TokenTree");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // Token
        // Token except delimiters
        self.lexer.peek();

        if let Ok(expr) = self.delim_token_tree() {
            return Ok(expr);
        }

        self.error(SyntaxError::ExpectedToken, &key)
    }

    // MacroInvocationSemi ::= SimplePath `!` `(` TokenTree* `)` `;`
    //                       | SimplePath `!` `[` TokenTree*`]` `;`
    //                       | SimplePath `!` `{` TokenTree* `}`
    fn macro_invocation_semi(&mut self) -> Result<CSTNode, Error> {
        let key = self.make_key("MacroInvocationSemi");
        match self.get_memo(&key) {
            MemoResult::Some(res) => return Ok(res),
            MemoResult::Recursive => return self.error(SyntaxError::Recursed, &key),
            MemoResult::None => self.write_memo(&key, None),
        };
        let mut node = CSTNode::new(CSTNodeKind::None, vec![]);

        // SimplePath
        let simple_path = self.simple_path()?;

        // `!`
        if !matches!(self.lexer.peek(), Token::Not) {
            return self.error(SyntaxError::ExpectedToken, &key);
        }

        // `(` TokenTree* `)` `;`
        if matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Parenthesis)
        ) {
        } else if matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Brackets)
        ) {
        } else if matches!(
            self.lexer.peek(),
            Token::LeftParenthesis(LeftParenthesis::Brace)
        ) {
        }

        self.error(SyntaxError::ExpectedToken, &key)
    }

    //
    //
    //

    fn make_factor(&self) -> CSTNode {
        let index = self.lexer.get_token_position();
        let token = self.lexer.peek();
        CSTNode::new(CSTNodeKind::Factor { token, index }, vec![])
    }

    fn make_factor_and_next(&mut self) -> CSTNode {
        let index = self.lexer.get_token_position();
        let token = self.lexer.next();
        CSTNode::new(CSTNodeKind::Factor { token, index }, vec![])
    }

    fn make_factor_and_next_glue(&mut self) -> CSTNode {
        let index = self.lexer.get_token_position();
        let token = self.lexer.next_glue();
        CSTNode::new(CSTNodeKind::Factor { token, index }, vec![])
    }

    fn make_operator(
        &mut self,
        token: Token,
        left: Option<Box<CSTNode>>,
        right: Option<Box<CSTNode>>,
    ) -> CSTNode {
        let index = self.lexer.get_token_position();
        CSTNode::new(
            CSTNodeKind::Operator {
                token,
                left,
                right,
                index,
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
            "Write Memo {} pos: {:?} token: {:?} \n",
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

        let Ok(_) = write!(log_file, "{}", self.log) else {
            println!("log output error!");
            return;
        };
    }
}
