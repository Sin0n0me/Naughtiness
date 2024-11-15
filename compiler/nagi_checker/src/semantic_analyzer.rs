use nagi_errors::*;
use nagi_syntax_tree::ast::*;
use nagi_syntax_tree::cst::*;
use nagi_syntax_tree::hst::*;
use nagi_syntax_tree::keywords::Keyword;
use nagi_syntax_tree::token::*;

use crate::type_checker::TypeChecker;
use crate::SymbolTreeNode;

use std::rc::Rc;

#[derive()]
pub struct SemanticAnalyzer {
    symbol_table: SymbolTreeNode,
    type_checker: TypeChecker,
}

// 今のところは型チェックと変数や関数の重複チェックのみ
impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTreeNode::new(),
            type_checker: TypeChecker::new(),
        }
    }

    pub fn semantic_analyze(&mut self, cst: &CSTNode) -> Result<ASTNode, Error> {
        let ast = match &cst.node_kind {
            CSTNodeKind::Crate {
                inner_attributes,
                items,
            } => {
                let mut inner_attribute = vec![];
                let mut item = vec![];

                for attribute in inner_attributes.iter() {
                    inner_attribute.push(analyze(attribute, &mut self.symbol_table)?);
                }

                for cst_item in items.iter() {
                    item.push(analyze(cst_item, &mut self.symbol_table)?);
                }

                ASTNode::new(ASTNodeKind::Crate {
                    inner_attribute,
                    item,
                })
            }
            _ => {
                return Err(Error {
                    error_kind: ErrorKind::Semantic(SemanticError::TODO),
                    error_text: format!(""),
                })
            }
        };

        //println!("{:#?}", self.symbol_table);

        Ok(ast)
    }

    pub fn type_check(&mut self) {}
}

fn analyze(cst: &CSTNode, symbol_tree: &mut SymbolTreeNode) -> Result<ASTNode, Error> {
    let ast = match &cst.node_kind {
        CSTNodeKind::Crate {
            inner_attributes:_,
            items:_,
        } => {
            panic!();
        }
        CSTNodeKind::Factor {
            token,
            row: _,
            column: _,
        } => ASTNode::new(ASTNodeKind::Factor {
            token: token.clone(),
        }),
        CSTNodeKind::Literal {
            literal,
            row: _,
            column: _,
        } => ASTNode::new(ASTNodeKind::Literal {
            literal: literal.clone(),
        }),
        CSTNodeKind::Operator {
            token,
            row: _,
            column: _,
        } => analyze_operator(cst, symbol_tree, token)?
                  ,
        CSTNodeKind::InnerAttribute {
            pound: _,
            exclamation: _,
            left_brackets: _,
            attribute,
            right_brackets: _,
        } => ASTNode::new(ASTNodeKind::InnerAttribute {
            attribute: Box::new(analyze(attribute, symbol_tree)?),
        }),
        CSTNodeKind::OuterAttribute {
            pound: _,
            left_brackets: _,
            attribute,
            right_brackets: _,
        } => ASTNode::new(ASTNodeKind::InnerAttribute {
            attribute: Box::new(analyze(attribute, symbol_tree)?),
        }),

        // Expression
        CSTNodeKind::Expression { expression } => ASTNode::new(ASTNodeKind::Expression {
            expression: Box::new(analyze(expression, symbol_tree)?),
        }),

        CSTNodeKind::ExpressionWithBlock { outer_attribute, expression_with_block } => {

            let mut ast_outer_attribute = vec![];
            for expr in outer_attribute.iter() {
                ast_outer_attribute.push(analyze(expr, symbol_tree)?);
            }
            let ast_expression_with_block = Box::new(analyze(expression_with_block, symbol_tree)?);

            ASTNode::new(ASTNodeKind::ExpressionWithBlock{
outer_attribute: ast_outer_attribute,
    expression_with_block: ast_expression_with_block,

            })
        }

        CSTNodeKind::ExpressionWithoutBlock { outer_attribute, expression } => {
           let mut ast_outer_attribute = vec![];
            for expr in outer_attribute.iter() {
                ast_outer_attribute.push(analyze(expr, symbol_tree)?);
            }
            let ast_expression = Box::new(analyze(expression, symbol_tree)?);

            ASTNode::new(ASTNodeKind::ExpressionWithoutBlock{
outer_attribute: ast_outer_attribute,
    expression: ast_expression,

            })


        }

        CSTNodeKind::LiteralExpression { literal } => analyze(literal, symbol_tree)?,

        CSTNodeKind::BlockExpression {
            left_brace: _,
            inner_attribute,
            statements,
            right_brace: _,
        } => {
            // ブロックの場合はネストしてから
            let Some(expr) = statements else {
                panic!();
            };
            // InnerAttribute
            let mut child = symbol_tree.add_child();
            let mut inner_attri = vec![];
            for attr in inner_attribute {
                inner_attri.push(analyze(attr, &mut child)?);
            }

            // Statement
            let mut statements = None;
            if let Ok(statement) = analyze(expr, &mut symbol_tree.add_child()) {
                statements = Some(Box::new(statement));
            }

            ASTNode::new(ASTNodeKind::BlockExpression {
                inner_attribute: inner_attri,
                statements,
            })
        }

        CSTNodeKind::PathExpression { path_in_expression } => {
            ASTNode::new(ASTNodeKind::PathExpression {
                expression: Box::new(analyze(path_in_expression, symbol_tree)?),
            })
        }

        CSTNodeKind::PathInExpression {
            path_separater: _,
            path_expr_segment,
            repeat_path_expr_segment,
        } => {
            let ast_path_expr_segment = Box::new(analyze(path_expr_segment, symbol_tree)?);
            let mut ast_repeat_path_expr_segment = vec![];
            for (_, expr) in repeat_path_expr_segment {
                ast_repeat_path_expr_segment.push(analyze(expr, symbol_tree)?);
            }

            ASTNode::new(ASTNodeKind::PathInExpression {
                path_expr_segment: ast_path_expr_segment,
                repeat_path_expr_segment: ast_repeat_path_expr_segment,
            })
        }

        // Function
        CSTNodeKind::Function {
            function_qualifiers,
            fn_keyword: _,
            identifier,
            generic_params,
            left_parenthesis: _,
            function_parameters,
            right_parenthesis: _,
            function_return_type,
            where_clause,
            block_expression_or_semicolon,
        } => {
            let ast_function_qualifiers = Box::new(analyze(function_qualifiers, symbol_tree)?);

            let CSTNodeKind::Factor {
                token,
                row: _,
                column: _,
            } = &identifier.node_kind
            else {
                panic!();
            };
            let Token::Identifier(ident) = token else {
                panic!();
            };

            let mut ast_generic_params = None;
            if let Some(generic_param) = generic_params {
                ast_generic_params = Some(Box::new(analyze(generic_param, symbol_tree)?));
            }

            let mut ast_function_parameters = None;
            if let Some(expr) = function_parameters {
                ast_function_parameters = Some(Box::new(analyze(expr, symbol_tree)?));
            }

            let mut ast_return_type = None;
            if let Some(return_type) = function_return_type {
                ast_return_type = Some(Box::new(analyze(return_type, symbol_tree)?));
            }

            // TODO 戻り値の型
            if !symbol_tree.insert_function(ident, None) {
                //panic!("`{}`関数はすでに定義されています", ident);
                return Err(Error{
error_kind: ErrorKind::Semantic(SemanticError::RedefinitionFunction),
error_text: format!("`{}`関数はすでに定義されています", ident)
                })
            }

            // BlockExpression内の定義をみていく
            let mut ast_block_expression = None;
            if let Ok(expr) = analyze(block_expression_or_semicolon, &mut symbol_tree.add_child()) {
                ast_block_expression = Some(Box::new(expr));
            }

            ASTNode::new(ASTNodeKind::Function {
                function_qualifiers: ast_function_qualifiers,
                identifier: ident.to_string(),
                generic_params: ast_generic_params,
                function_parameters: ast_function_parameters,
                function_return_type: ast_return_type,
                where_clause: None,
                block_expression: ast_block_expression,
            })
        }

        CSTNodeKind::FunctionQualifiers {
            const_keyword,
            async_keyword,
            item_safety,
            extern_keyword,
            abi,
        } => {
            let mut ast_item_safety = None;
            if let Some(expr) = item_safety {
                //ast_item_safety = Some();
            }

            let mut ast_abi = None;
            if let Some(expr) = abi {
                //ast_abi = Some();
            }

            ASTNode::new(ASTNodeKind::FunctionQualifiers {
                const_keyword: const_keyword.is_some(),
                async_keyword: async_keyword.is_some(),
                item_safety: ast_item_safety,
                extern_keyword: extern_keyword.is_some(),
                abi: ast_abi,
            })
        }

        // Struct
        CSTNodeKind::StructExpression { expression } => {
            panic!();
        }

        // Pattern
        CSTNodeKind::IdentifierPattern {
            ref_keyword,
            mut_keyword,
            identifier,
            at_symbol:_,
            pattern_no_top_alt,
        } => analyze_identifier_pattern(symbol_tree, ref_keyword.is_some(), mut_keyword.is_some(), &identifier.node_kind,pattern_no_top_alt)?
        ,

        // Statements
        CSTNodeKind::Statements => {
            let mut statements = vec![];
            for child in cst.children.iter() {
                statements.push(analyze(child, symbol_tree)?);
            }

            ASTNode::new(ASTNodeKind::Statements { statements })
        }

        CSTNodeKind::Statement { statement } => ASTNode::new(ASTNodeKind::Statement {
            statement: Some(Box::new(analyze(statement, symbol_tree)?)),
        }),

        CSTNodeKind::LetStatement {
            outer_attribute,
            rarity,
            pattern_no_top_alt,
            colon: _,
            type_expression,
            equal: _,
            expression,
            else_keyword: _,
            block_expression,
            semicolon: _,
        } => {
            analyze_let_statement(symbol_tree, outer_attribute,
                rarity,pattern_no_top_alt,type_expression,
                expression,block_expression)?
        }

        _ => panic!("{:?}", cst.node_kind),
    };

    Ok(ast)
}

fn analyze_operator(
    cst: &CSTNode,
    symbol_tree: &mut SymbolTreeNode,
    token: &Token,
) -> Result<ASTNode, Error> {
    let Some(left_child) = cst.children.first() else {
        panic!();
    };
    let left = Box::new(analyze(left_child, symbol_tree)?);

    let Some(right_child) = cst.children.first() else {
        panic!();
    };
    let right = Box::new(analyze(right_child, symbol_tree)?);

    let operator = match token {
        Token::Plus => BinaryOperator::Add,
        Token::Minus => BinaryOperator::Sub,
        Token::Star => BinaryOperator::Mul,
        Token::Slash => BinaryOperator::Div,
        Token::Percent => BinaryOperator::Mod,
        Token::Caret => BinaryOperator::Xor,
        Token::LeftShift => BinaryOperator::LeftShift,
        Token::RightShift => BinaryOperator::RightShiht,
        _ => panic!(),
    };

    Ok(ASTNode::new(ASTNodeKind::BinaryOperator {
        operator,
        left,
        right,
    }))
}

fn analyze_identifier_pattern(
    symbol_tree: &mut SymbolTreeNode,
    ref_keyword: bool,
    mut_keyword: bool,
    node_kind: &CSTNodeKind,
    pattern_no_top_alt: &Option<Box<CSTNode>>,
) -> Result<ASTNode, Error> {
    let CSTNodeKind::Factor {
        token,
        row: _,
        column: _,
    } = node_kind
    else {
        panic!();
    };
    let Token::Identifier(ident) = token else {
        panic!();
    };

    let mut ast_pattern_no_top_alt = None;
    if let Some(expr) = pattern_no_top_alt {
        ast_pattern_no_top_alt = Some(Box::new(analyze(expr, symbol_tree)?));
    }

    Ok(ASTNode::new(ASTNodeKind::IdentifierPattern {
        ref_keyword,
        mut_keyword,
        identifier: ident.to_string(),
        pattern_no_top_alt: ast_pattern_no_top_alt,
    }))
}

fn analyze_let_statement(
    symbol_tree: &mut SymbolTreeNode,
    outer_attribute: &Vec<CSTNode>,
    rarity: &CSTNode,
    pattern_no_top_alt: &CSTNode,
    type_expression: &Option<Box<CSTNode>>,
    expression: &Option<Box<CSTNode>>,
    block_expression: &Option<Box<CSTNode>>,
) -> Result<ASTNode, Error> {
    let mut ast_outer_attribute = vec![];
    for expr in outer_attribute {
        ast_outer_attribute.push(analyze(expr, symbol_tree)?);
    }

    let CSTNodeKind::Factor {
        token,
        row: _,
        column: _,
    } = &rarity.node_kind
    else {
        panic!();
    };
    let ast_rarity = match token {
        Token::Keyword(Keyword::Let) => Rarity::Let,
        Token::Keyword(Keyword::Ur) => Rarity::Ur,
        Token::Keyword(Keyword::Sr) => Rarity::Sr,
        Token::Keyword(Keyword::Nr) => Rarity::Nr,
        _ => panic!("Unknown Rarity"),
    };

    let ast_pattern_no_top_alt = Box::new(analyze(pattern_no_top_alt, symbol_tree)?);

    let mut ast_type_expression = None;
    if let Some(expr) = type_expression {
        ast_type_expression = Some(Box::new(analyze(expr, symbol_tree)?));
    }

    let mut ast_expression = None;
    if let Some(expr) = expression {
        ast_expression = Some(Box::new(analyze(expr, symbol_tree)?));
    }

    let mut ast_block_expression = None;
    if let Some(expr) = block_expression {
        ast_block_expression = Some(Box::new(analyze(expr, symbol_tree)?));
    }

    Ok(ASTNode::new(ASTNodeKind::LetStatement {
        outer_attribute: ast_outer_attribute,
        rarity: ast_rarity,
        pattern_no_top_alt: ast_pattern_no_top_alt,
        type_expression: ast_type_expression,
        expression: ast_expression,
        block_expression: ast_block_expression,
    }))
}
