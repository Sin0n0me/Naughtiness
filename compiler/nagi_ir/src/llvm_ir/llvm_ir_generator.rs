use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::values::*;
use inkwell::OptimizationLevel;
use nagi_syntax_tree::ast::*;
use nagi_syntax_tree::token::*;
use std::collections::HashMap;

enum Value<'ctx> {
    Int(IntValue<'ctx>),
    Float(FloatValue<'ctx>),
}

pub struct LLVMGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> LLVMGenerator<'ctx> {
    pub fn new(context: &'ctx Context) -> LLVMGenerator<'ctx> {
        let module = context.create_module("nagi");
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
        }
    }

    pub fn compile_ast_to_llvm(&self, ast: &ASTNode) -> Option<Value<'ctx>> {
        let v = match &ast.node_kind {
            ASTNodeKind::Literal { literal } => self.compile_literal(literal),
            ASTNodeKind::BinaryOperator {
                operator,
                left,
                right,
            } => self.compile_operator(operator, left, right),

            ASTNodeKind::Statement { statement } => {
                self.compile_ast_to_llvm(statement.as_ref().unwrap())?
            }
            ASTNodeKind::Statements { statements } => {
                for statement in statements {
                    self.compile_ast_to_llvm(statement);
                }

                return None;
            }
            ASTNodeKind::IfExpression {
                expression,
                block_expression,
                else_expression,
            } => {
                self.compile_if_statement(expression, block_expression, else_expression.as_deref());
                return None;
            }

            ASTNodeKind::LoopExpression {
                loop_label,
                loop_expression,
            } => {
                self.compile_loop(loop_expression);
                return None;
            }
            _ => panic!(),
        };

        Some(v)
    }

    fn compile_literal(&self, literal: &Literal) -> Value<'ctx> {
        match literal.literal_kind {
            LiteralKind::Integer => Value::Int(self.compile_integer(literal)),
            LiteralKind::Float => Value::Float(self.compile_float(literal)),

            _ => panic!(),
        }
    }

    fn compile_integer(&self, literal: &Literal) -> IntValue<'ctx> {
        let i32_type = self.context.i32_type();

        i32_type.const_int(literal.symbol.as_str().parse::<u64>().unwrap(), false)
    }

    fn compile_float(&self, literal: &Literal) -> FloatValue<'ctx> {
        let f32_type = self.context.f32_type();

        f32_type.const_float(literal.symbol.as_str().parse::<f64>().unwrap())
    }

    fn compile_operator(
        &self,
        operator: &BinaryOperator,
        left: &ASTNode,
        right: &ASTNode,
    ) -> Value<'ctx> {
        let lhs_val = self.compile_ast_to_llvm(left).unwrap();
        let rhs_val = self.compile_ast_to_llvm(right).unwrap();

        match (lhs_val, rhs_val) {
            (Value::Int(left_int), Value::Int(right_int)) => {
                Value::Int(self.compile_integer_operator(operator, left_int, right_int))
            }
            (Value::Float(left_float), Value::Float(right_float)) => {
                Value::Float(self.compile_float_operator(operator, left_float, right_float))
            }
            _ => panic!(),
        }
    }

    fn compile_integer_operator(
        &self,
        operator: &BinaryOperator,
        lhs_val: IntValue<'ctx>,
        rhs_val: IntValue<'ctx>,
    ) -> IntValue<'ctx> {
        match operator {
            BinaryOperator::Add => self.builder.build_int_add(lhs_val, rhs_val, "addtmp"),
            BinaryOperator::Sub => self.builder.build_int_sub(lhs_val, rhs_val, "subtmp"),
            BinaryOperator::Mul => self.builder.build_int_mul(lhs_val, rhs_val, "multmp"),
            BinaryOperator::Div => self
                .builder
                .build_int_signed_div(lhs_val, rhs_val, "divtmp"),
            BinaryOperator::And => self.builder.build_and(lhs_val, rhs_val, "andtmp"),
            BinaryOperator::Or => self.builder.build_or(lhs_val, rhs_val, "ortmp"),
            BinaryOperator::Xor => self.builder.build_xor(lhs_val, rhs_val, "xortmp"),
            _ => panic!(),
        }
        .unwrap()
    }

    fn compile_float_operator(
        &self,
        operator: &BinaryOperator,
        lhs_val: FloatValue<'ctx>,
        rhs_val: FloatValue<'ctx>,
    ) -> FloatValue<'ctx> {
        match operator {
            BinaryOperator::Add => self.builder.build_float_add(lhs_val, rhs_val, "addtmp"),
            BinaryOperator::Sub => self.builder.build_float_sub(lhs_val, rhs_val, "subtmp"),
            BinaryOperator::Mul => self.builder.build_float_mul(lhs_val, rhs_val, "multmp"),
            BinaryOperator::Div => self.builder.build_float_div(lhs_val, rhs_val, "divtmp"),
            _ => panic!(),
        }
        .unwrap()
    }

    fn compile_if_statement(
        &self,
        expression: &ASTNode,
        block_expression: &ASTNode,
        else_expression: Option<&ASTNode>,
    ) {
        // 条件式が最終的にintを返さなければエラー
        let Some(condition) = self.compile_ast_to_llvm(expression) else {
            panic!();
        };
        let Value::Int(condition_value) = condition else {
            panic!();
        };

        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        let then_block = self.context.append_basic_block(function, "then_block");
        let merge_block = self.context.append_basic_block(function, "if");

        // else ブロック判定
        if let Some(else_expr) = else_expression {
            let else_block = self.context.append_basic_block(function, "else_block");
            self.builder
                .build_conditional_branch(condition_value, then_block, else_block);

            // then
            self.builder.position_at_end(then_block);
            self.compile_ast_to_llvm(block_expression);
            self.builder.build_unconditional_branch(merge_block);

            // else
            self.builder.position_at_end(else_block);
            self.compile_ast_to_llvm(else_expr);
            self.builder.build_unconditional_branch(merge_block);
        } else {
            self.builder
                .build_conditional_branch(condition_value, then_block, merge_block);

            // then
            self.builder.position_at_end(then_block);
            self.compile_ast_to_llvm(block_expression);
            self.builder.build_unconditional_branch(merge_block);
        }

        self.builder.position_at_end(merge_block);
    }

    fn compile_loop(&self, loop_expression: &ASTNode) {
        match &loop_expression.node_kind {
            ASTNodeKind::InfiniteLoopExpression { block_expression } => {
                self.compile_infinite_loop(&block_expression)
            }
            ASTNodeKind::PredicateLoopExpression {
                expression,
                block_expression,
            } => self.compile_predicate_loop(&expression, &block_expression),

            _ => panic!(),
        }
    }

    fn compile_infinite_loop(&self, block_expression: &ASTNode) {
        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        let loop_entry = self.context.append_basic_block(function, "loop_entry");
        let loop_block = self.context.append_basic_block(function, "loop_body");

        self.builder.position_at_end(loop_entry);
        self.builder.build_unconditional_branch(loop_block);

        self.builder.position_at_end(loop_block);
        self.compile_ast_to_llvm(block_expression);
        self.builder.build_unconditional_branch(loop_block);
    }

    fn compile_predicate_loop(&self, expression: &ASTNode, block_expression: &ASTNode) {
        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        let condition_block = self.context.append_basic_block(function, "loop_condition");
        let body_block = self.context.append_basic_block(function, "loop_body");
        let end_block = self.context.append_basic_block(function, "loop_end");

        // 条件ブロックへのジャンプ
        self.builder.build_unconditional_branch(condition_block);

        self.builder.position_at_end(condition_block);
        let Some(condition_value) = self.compile_ast_to_llvm(block_expression) else {
            panic!();
        };
        let Value::Int(condition) = condition_value else {
            panic!("Condition must return an integer");
        };

        self.builder
            .build_conditional_branch(condition, body_block, end_block);

        self.builder.position_at_end(body_block);
        self.compile_ast_to_llvm(block_expression);
        self.builder.build_unconditional_branch(condition_block);

        self.builder.position_at_end(end_block);
    }

    fn compile_function() {}
}
