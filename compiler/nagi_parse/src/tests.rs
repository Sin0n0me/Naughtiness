#[cfg(test)]
mod test {
    use crate::cst_parse::cst_parser::CSTParser;
    use nagi_lexer::lexer::Lexer;
    use nagi_syntax_tree::cst::{CSTNode, CSTNodeKind};
    use nagi_syntax_tree::token;

    fn parse_cst(code: &str, tree: CSTNode) {
        let mut lexer = Lexer::new(code);
        let mut parser = CSTParser::new(&lexer.tokenize());

        match parser.parse() {
            Ok(cst) => {
                if cst != tree {
                    cst.write_cst("tree.json");
                }
                assert_eq!(cst, tree);
            }
            Err(_) => {
                assert!(false);
            }
        }
    }

    fn make_literal_expression(num: i128, row: usize, column: usize) -> CSTNode {
        CSTNode::new(
            CSTNodeKind::ExpressionWithoutBlock {
                outer_attribute: vec![],
                expression: Box::new(CSTNode::new(
                    CSTNodeKind::Literal {
                        literal: token::Literal::new(
                            token::LiteralKind::Integer,
                            num.to_string().as_str(),
                        ),
                        row,
                        column,
                    },
                    vec![],
                )),
            },
            vec![],
        )
    }

    #[test]
    fn check_literal_expression() {}

    #[test]
    fn check_path_expression() {}

    // 四則演算など
    #[test]
    fn check_operator_expression() {
        parse_cst(
            "1 + 2",
            CSTNode::new(
                CSTNodeKind::ExpressionWithoutBlock {
                    outer_attribute: vec![],
                    expression: Box::new(CSTNode::new(
                        CSTNodeKind::Factor {
                            token: token::Token::Plus,
                            row: 1,
                            column: 3,
                        },
                        vec![
                            make_literal_expression(1, 1, 1),
                            make_literal_expression(2, 1, 5),
                        ],
                    )),
                },
                vec![],
            ),
        );

        parse_cst(
            "3 - 1",
            CSTNode::new(
                CSTNodeKind::ExpressionWithoutBlock {
                    outer_attribute: vec![],
                    expression: Box::new(CSTNode::new(
                        CSTNodeKind::Factor {
                            token: token::Token::Minus,
                            row: 1,
                            column: 3,
                        },
                        vec![
                            make_literal_expression(3, 1, 1),
                            make_literal_expression(1, 1, 5),
                        ],
                    )),
                },
                vec![],
            ),
        );

        parse_cst(
            "6 / 2",
            CSTNode::new(
                CSTNodeKind::ExpressionWithoutBlock {
                    outer_attribute: vec![],
                    expression: Box::new(CSTNode::new(
                        CSTNodeKind::Factor {
                            token: token::Token::Slash,
                            row: 1,
                            column: 3,
                        },
                        vec![
                            make_literal_expression(6, 1, 1),
                            make_literal_expression(2, 1, 5),
                        ],
                    )),
                },
                vec![],
            ),
        );

        parse_cst(
            "5 * 6",
            CSTNode::new(
                CSTNodeKind::ExpressionWithoutBlock {
                    outer_attribute: vec![],
                    expression: Box::new(CSTNode::new(
                        CSTNodeKind::Factor {
                            token: token::Token::Star,
                            row: 1,
                            column: 3,
                        },
                        vec![
                            make_literal_expression(5, 1, 1),
                            make_literal_expression(6, 1, 5),
                        ],
                    )),
                },
                vec![],
            ),
        );

        //parse_cst("12 * 23 - 32 / 16 + 90");
    }

    #[test]
    fn check_call_expression() {}

    #[test]
    fn check_method_call_expression() {}

    #[test]
    fn check_field_expression() {}

    #[test]
    fn check_async_block_expression() {}

    #[test]
    fn check_continue_expression() {}

    #[test]
    fn check_break_expression() {}

    #[test]
    fn check_range_expression() {}

    #[test]
    fn check_return_expression() {}

    #[test]
    fn check_underscore_expression() {}

    #[test]
    fn check_block_expression() {
        parse_cst(
            "fn add() { sr hoge = 100 * 10;   }",
            make_literal_expression(1, 1, 1),
        );
    }

    #[test]
    fn check_const_block_expression() {}

    #[test]
    fn check_unsafe_block_expression() {}

    #[test]
    fn check_loop_expression() {}

    #[test]
    fn check_if_expression() {}

    #[test]
    fn check_if_let_expression() {}

    #[test]
    fn check_match_expression() {}
}
