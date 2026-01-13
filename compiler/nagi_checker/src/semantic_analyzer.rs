use nagi_errors::*;
use nagi_syntax_tree::ast::*;
use nagi_syntax_tree::cst::*;
use nagi_syntax_tree::hst::*;
use nagi_syntax_tree::keywords::Keyword;
use nagi_syntax_tree::token::*;

use crate::type_checker::TypeChecker;
use crate::SymbolTreeNode;

const todo_command_option: bool = false;

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
            inner_attributes,
            items,
        } => analyze_crate(symbol_tree, inner_attributes, items)?,

        CSTNodeKind::Factor { token, index: _ } => ASTNode::new(ASTNodeKind::Factor {
            token: token.clone(),
        }),

        CSTNodeKind::Literal { literal, index: _ } => ASTNode::new(ASTNodeKind::Literal {
            literal: literal.clone(),
        }),

        CSTNodeKind::Operator {
            token,
            left,
            right,
            index: _,
        } => analyze_operator(symbol_tree, token, left.as_deref(), right.as_deref())?,

        CSTNodeKind::InnerAttribute { attribute } => ASTNode::new(ASTNodeKind::InnerAttribute {
            attribute: Box::new(analyze(attribute, symbol_tree)?),
        }),

        CSTNodeKind::OuterAttribute { attribute } => ASTNode::new(ASTNodeKind::OuterAttribute {
            attribute: Box::new(analyze(attribute, symbol_tree)?),
        }),

        CSTNodeKind::VisItem { visibility, item } => {
            analyze_vis_item(symbol_tree, visibility.as_deref().cloned(), item)?
        }

        CSTNodeKind::Visibility { pub_keyword } => {
            ASTNode::new(ASTNodeKind::Visibility { path: None })
        }

        CSTNodeKind::Expression { expression } => ASTNode::new(ASTNodeKind::Expression {
            expression: Box::new(analyze(expression, symbol_tree)?),
        }),

        CSTNodeKind::ExpressionWithBlock {
            outer_attribute,
            expression_with_block,
        } => analyze_expression_with_block(symbol_tree, outer_attribute, expression_with_block)?,

        CSTNodeKind::ExpressionWithoutBlock {
            outer_attribute,
            expression,
        } => analyze_expression_without_block(symbol_tree, outer_attribute, expression)?,

        CSTNodeKind::LiteralExpression { literal } => analyze(literal, symbol_tree)?,

        CSTNodeKind::BlockExpression {
            inner_attribute,
            statements,
        } => {
            // InnerAttribute
            let mut child = symbol_tree.add_child();
            let mut ast_inner_attribute = vec![];
            for attr in inner_attribute {
                ast_inner_attribute.push(analyze(attr, &mut child)?);
            }

            let Some(expr) = statements else {
                return Ok(ASTNode::new(ASTNodeKind::BlockExpression {
                    inner_attribute: ast_inner_attribute,
                    statements: None,
                }));
            };

            // Statement
            let mut statements = None;
            if let Ok(statement) = analyze(expr, &mut symbol_tree.add_child()) {
                statements = Some(Box::new(statement));
            }

            ASTNode::new(ASTNodeKind::BlockExpression {
                inner_attribute: ast_inner_attribute,
                statements,
            })
        }

        CSTNodeKind::PathExpression { path_in_expression } => {
            ASTNode::new(ASTNodeKind::PathExpression {
                expression: Box::new(analyze(path_in_expression, symbol_tree)?),
            })
        }

        CSTNodeKind::PathInExpression { path_expr_segment } => {
            let mut ast_path_expr_segment = vec![];
            for expr in path_expr_segment {
                ast_path_expr_segment.push(analyze(expr, symbol_tree)?);
            }

            ASTNode::new(ASTNodeKind::PathInExpression {
                path_expr_segment: ast_path_expr_segment,
            })
        }

        // Function
        CSTNodeKind::Function {
            function_qualifiers,
            identifier,
            generic_params,
            function_parameters,
            function_return_type,
            where_clause,
            block_expression_or_semicolon,
        } => analyze_function(
            symbol_tree,
            function_qualifiers,
            identifier,
            generic_params.as_deref(),
            function_parameters.as_deref(),
            function_return_type.as_deref(),
            where_clause.as_deref(),
            block_expression_or_semicolon,
        )?,

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
        CSTNodeKind::PatternNoTopAlt { pattern } => ASTNode::new(ASTNodeKind::PatternNoTopAlt {
            pattern: Box::new(analyze(pattern, symbol_tree)?),
        }),

        CSTNodeKind::PatternWithoutRange { pattern } => {
            ASTNode::new(ASTNodeKind::PatternWithoutRange {
                pattern: Box::new(analyze(pattern, symbol_tree)?),
            })
        }

        CSTNodeKind::IdentifierPattern {
            ref_keyword,
            mut_keyword,
            identifier,
            pattern_no_top_alt,
        } => analyze_identifier_pattern(
            symbol_tree,
            ref_keyword.is_some(),
            mut_keyword.is_some(),
            &identifier.node_kind,
            pattern_no_top_alt,
        )?,

        // Statements
        CSTNodeKind::Statements { statements } => {
            let mut ast_statements = vec![];
            for cst_statement in statements.iter() {
                ast_statements.push(analyze(cst_statement, symbol_tree)?);
            }

            ASTNode::new(ASTNodeKind::Statements {
                statements: ast_statements,
            })
        }

        CSTNodeKind::Statement { statement } => ASTNode::new(ASTNodeKind::Statement {
            statement: Some(Box::new(analyze(statement, symbol_tree)?)),
        }),

        CSTNodeKind::ExpressionStatement { expression } => analyze(expression, symbol_tree)?,

        CSTNodeKind::LetStatement {
            outer_attribute,
            rarity,
            pattern_no_top_alt,
            type_expression,
            expression,
            block_expression,
        } => analyze_let_statement(
            symbol_tree,
            outer_attribute,
            rarity,
            pattern_no_top_alt,
            type_expression,
            expression,
            block_expression,
        )?,

        CSTNodeKind::IfExpression {
            expression,
            block_expression,
            else_expression,
        } => {
            let expression = Box::new(analyze(expression, symbol_tree)?);
            let block_expression = Box::new(analyze(block_expression, symbol_tree)?);
            let else_expression = if let Some(expr) = else_expression {
                Some(Box::new(analyze(expr, symbol_tree)?))
            } else {
                None
            };

            ASTNode::new(ASTNodeKind::IfExpression {
                expression,
                block_expression,
                else_expression,
            })
        }

        CSTNodeKind::LoopExpression {
            loop_label: _,
            loop_expression,
        } => ASTNode::new(ASTNodeKind::LoopExpression {
            loop_label: None, // TODO
            loop_expression: Box::new(analyze(loop_expression, symbol_tree)?),
        }),

        CSTNodeKind::InfiniteLoopExpression { block_expression } => {
            ASTNode::new(ASTNodeKind::InfiniteLoopExpression {
                block_expression: Box::new(analyze(block_expression, symbol_tree)?),
            })
        }

        CSTNodeKind::PredicateLoopExpression {
            expression,
            block_expression,
        } => ASTNode::new(ASTNodeKind::PredicateLoopExpression {
            expression: Box::new(analyze(expression, symbol_tree)?),
            block_expression: Box::new(analyze(block_expression, symbol_tree)?),
        }),

        CSTNodeKind::PathIdentSegment { path_ident_segment } => {
            analyze(path_ident_segment, symbol_tree)?
        }

        CSTNodeKind::PathExprSegment {
            path_ident_segment,
            generic_args,
        } => ASTNode::new(ASTNodeKind::PathExprSegment {
            path_ident_segment: Box::new(analyze(path_ident_segment, symbol_tree)?),
            generic_args: None, // TODO Box::new(analyze(block_expression, symbol_tree)?),
        }),

        _ => panic!("{:#?}", cst.node_kind),
    };

    Ok(ast)
}

fn analyze_crate(
    symbol_tree: &mut SymbolTreeNode,
    inner_attributes: &[CSTNode],
    items: &[CSTNode],
) -> Result<ASTNode, Error> {
    let mut inner_attribute = vec![];
    for attrubute in inner_attributes {
        inner_attribute.push(analyze(attrubute, symbol_tree)?);
    }

    let mut item = vec![];
    for cst_item in items {
        item.push(analyze(cst_item, symbol_tree)?);
    }

    Ok(ASTNode::new(ASTNodeKind::Crate {
        inner_attribute,
        item,
    }))
}

fn analyze_vis_item(
    symbol_tree: &mut SymbolTreeNode,
    cst_visibility: Option<CSTNode>,
    cst_item: &CSTNode,
) -> Result<ASTNode, Error> {
    let mut visibility = None;
    if let Some(vis) = cst_visibility {
        visibility = Some(Box::new(analyze(&vis, symbol_tree)?));
    }

    let item = Box::new(analyze(cst_item, symbol_tree)?);

    Ok(ASTNode::new(ASTNodeKind::VisItem { visibility, item }))
}

fn analyze_expression_with_block(
    symbol_tree: &mut SymbolTreeNode,
    outer_attribute: &[CSTNode],
    expression_with_block: &CSTNode,
) -> Result<ASTNode, Error> {
    let mut ast_outer_attribute = vec![];
    for expr in outer_attribute.iter() {
        ast_outer_attribute.push(analyze(expr, symbol_tree)?);
    }
    let ast_expression_with_block = Box::new(analyze(expression_with_block, symbol_tree)?);

    Ok(ASTNode::new(ASTNodeKind::ExpressionWithBlock {
        outer_attribute: ast_outer_attribute,
        expression_with_block: ast_expression_with_block,
    }))
}

fn analyze_expression_without_block(
    symbol_tree: &mut SymbolTreeNode,
    outer_attribute: &[CSTNode],
    expression: &CSTNode,
) -> Result<ASTNode, Error> {
    let mut ast_outer_attribute = vec![];
    for expr in outer_attribute.iter() {
        ast_outer_attribute.push(analyze(expr, symbol_tree)?);
    }
    let ast_expression = Box::new(analyze(expression, symbol_tree)?);

    Ok(ASTNode::new(ASTNodeKind::ExpressionWithoutBlock {
        outer_attribute: ast_outer_attribute,
        expression: ast_expression,
    }))
}

fn analyze_operator(
    symbol_tree: &mut SymbolTreeNode,
    token: &Token,
    left_cst: Option<&CSTNode>,
    right_cst: Option<&CSTNode>,
) -> Result<ASTNode, Error> {
    if left_cst.is_none() || right_cst.is_none() {
        return Err(Error {
            error_kind: ErrorKind::Semantic(SemanticError::TODO),
            error_text: String::from(""),
        });
    }

    let operator = match token {
        Token::Plus => BinaryOperator::Add,
        Token::Minus => BinaryOperator::Sub,
        Token::Star => BinaryOperator::Mul,
        Token::Slash => BinaryOperator::Div,
        Token::Percent => BinaryOperator::Mod,
        Token::Caret => BinaryOperator::Xor,
        Token::LeftShift => BinaryOperator::LeftShift,
        Token::RightShift => BinaryOperator::RightShiht,
        Token::And => BinaryOperator::And,
        Token::Or => BinaryOperator::Or,
        Token::PlusEqual => BinaryOperator::AddAssign,
        Token::MinusEqual => BinaryOperator::SubAssign,
        Token::StarEqual => BinaryOperator::MulAssign,
        Token::SlashEqual => BinaryOperator::DivAssign,
        Token::AndEqual => BinaryOperator::AndAssign,
        Token::OrEqual => BinaryOperator::OrAssign,
        Token::OrOr => BinaryOperator::ConditionOr,
        Token::AndAnd => BinaryOperator::ConditionAnd,
        Token::EqualEqual => BinaryOperator::ConditionCompare,
        Token::NotEqual => BinaryOperator::ConditionNotCompare,
        Token::GreaterThan => BinaryOperator::ConditionGreaterThan,
        Token::GreaterThanOrEqual => BinaryOperator::ConditionGreaterThanEqual,
        Token::LessThan => BinaryOperator::ConditionLessThan,
        Token::LessThanOrEqual => BinaryOperator::ConditionLessThanEquel,
        _ => panic!(),
    };

    // TODO
    let right = Box::new(_todo_naming_this_function(
        symbol_tree,
        &operator,
        right_cst.as_ref().unwrap(),
    )?);

    let left = match operator {
        // 代入系は右側だけ
        BinaryOperator::Equal
        | BinaryOperator::AddAssign
        | BinaryOperator::SubAssign
        | BinaryOperator::MulAssign
        | BinaryOperator::DivAssign
        | BinaryOperator::ModAssign
        | BinaryOperator::XorAssign
        | BinaryOperator::OrAssign
        | BinaryOperator::AndAssign
        | BinaryOperator::LeftShiftAssign
        | BinaryOperator::RightShihtAssign => {
            let left = Box::new(analyze(left_cst.as_ref().unwrap(), symbol_tree)?);

            if todo_command_option {
                return Ok(ASTNode::new(ASTNodeKind::Statements {
                    statements: vec![
                        // そのまま計算するとバグるので一度元に戻して再度値を変える
                        ASTNode::new(ASTNodeKind::BinaryOperator {
                            operator: BinaryOperator::Equal,
                            left: left.clone(),
                            right: Box::new(ASTNode::new(ASTNodeKind::BinaryOperator {
                                operator: BinaryOperator::Xor,
                                left: left.clone(),
                                right: Box::new(add_xor(ASTNode::new(ASTNodeKind::Literal {
                                    literal: Literal::new(LiteralKind::Integer, "0"),
                                }))),
                            })),
                        }),
                        ASTNode::new(ASTNodeKind::BinaryOperator {
                            operator,
                            left: left.clone(),
                            right,
                        }),
                        ASTNode::new(ASTNodeKind::BinaryOperator {
                            operator: BinaryOperator::XorAssign,
                            left: left.clone(),
                            right: Box::new(add_xor(ASTNode::new(ASTNodeKind::Literal {
                                literal: Literal::new(LiteralKind::Integer, "0"),
                            }))),
                        }),
                    ],
                }));
            } else {
                return Ok(ASTNode::new(ASTNodeKind::BinaryOperator {
                    operator,
                    left: left.clone(),
                    right,
                }));
            }
        }

        _ => Box::new(_todo_naming_this_function(
            symbol_tree,
            &operator,
            left_cst.as_ref().unwrap(),
        )?),
    };

    Ok(ASTNode::new(ASTNodeKind::BinaryOperator {
        operator,
        left,
        right,
    }))
}

// TODO TODO TODO
// hoge > 0 とかの片方がただの定数だと (hoge ^ fuga) > (0 ~ fuga) になるのでそれを防ぐ関数
fn _todo_naming_this_function(
    symbol_tree: &mut SymbolTreeNode,
    operator: &BinaryOperator,
    cst: &CSTNode,
) -> Result<ASTNode, Error> {
    let ast = analyze(cst, symbol_tree)?;

    if matches!(
        operator,
        BinaryOperator::ConditionCompare
            | BinaryOperator::ConditionNotCompare
            | BinaryOperator::ConditionAnd
            | BinaryOperator::ConditionOr
            | BinaryOperator::ConditionLessThan
            | BinaryOperator::ConditionGreaterThan
            | BinaryOperator::ConditionLessThanEquel
            | BinaryOperator::ConditionGreaterThanEqual
    ) {
        return Ok(ast);
    }

    // TODO!!!!!!!!!!!!
    if let ASTNodeKind::Expression { expression } = &ast.node_kind {
        if let ASTNodeKind::ExpressionWithoutBlock {
            outer_attribute,
            expression,
        } = &expression.node_kind
        {
            if let ASTNodeKind::Literal { literal } = &expression.node_kind {
                return Ok(ast);
            }
        }
    }

    Ok(add_xor(ast))
}

fn analyze_function(
    symbol_tree: &mut SymbolTreeNode,
    function_qualifiers: &CSTNode,
    identifier: &CSTNode,
    generic_params: Option<&CSTNode>,
    function_parameters: Option<&CSTNode>,
    function_return_type: Option<&CSTNode>,
    where_clause: Option<&CSTNode>,
    block_expression_or_semicolon: &CSTNode,
) -> Result<ASTNode, Error> {
    let ast_function_qualifiers = Box::new(analyze(function_qualifiers, symbol_tree)?);

    let CSTNodeKind::Factor { token, index: _ } = &identifier.node_kind else {
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
        return Err(Error {
            error_kind: ErrorKind::Semantic(SemanticError::RedefinitionFunction),
            error_text: format!("`{}`関数はすでに定義されています", ident),
        });
    }

    // BlockExpression内の定義をみていく
    let mut ast_block_expression = None;
    if let Ok(expr) = analyze(block_expression_or_semicolon, &mut symbol_tree.add_child()) {
        ast_block_expression = Some(Box::new(expr));
    }

    Ok(ASTNode::new(ASTNodeKind::Function {
        function_qualifiers: ast_function_qualifiers,
        identifier: ident.to_string(),
        generic_params: ast_generic_params,
        function_parameters: ast_function_parameters,
        function_return_type: ast_return_type,
        where_clause: None,
        block_expression: ast_block_expression,
    }))
}

fn analyze_identifier_pattern(
    symbol_tree: &mut SymbolTreeNode,
    ref_keyword: bool,
    mut_keyword: bool,
    node_kind: &CSTNodeKind,
    pattern_no_top_alt: &Option<Box<CSTNode>>,
) -> Result<ASTNode, Error> {
    let CSTNodeKind::Factor { token, index: _ } = node_kind else {
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
    outer_attribute: &[CSTNode],
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

    let CSTNodeKind::Factor { token, index: _ } = &rarity.node_kind else {
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
        // TODO 別の場所に移す
        let ast = add_xor(analyze(expr, symbol_tree)?);
        ast_expression = Some(Box::new(ast));
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

fn add_xor(ast: ASTNode) -> ASTNode {
    if todo_command_option {
        ASTNode::new(ASTNodeKind::BinaryOperator {
            operator: BinaryOperator::Xor,
            left: Box::new(ast),
            right: Box::new(ASTNode::new(ASTNodeKind::Literal {
                literal: Literal::new(LiteralKind::Integer, "12648430"),
            })),
        })
    } else {
        ast
    }
}
