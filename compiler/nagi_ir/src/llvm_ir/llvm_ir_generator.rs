use anyhow::{anyhow, Ok, Result};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::*;
use inkwell::types::*;
use inkwell::values::*;
use inkwell::{FloatPredicate, IntPredicate, OptimizationLevel};
use nagi_syntax_tree::ast::*;
use nagi_syntax_tree::token::*;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy)]
enum Value<'ctx> {
    Bool(IntValue<'ctx>),
    Int(IntValue<'ctx>),
    Float(FloatValue<'ctx>),
    Pointer(PointerValue<'ctx>),
    Struct(StructValue<'ctx>),
    Array(ArrayValue<'ctx>),
}

#[derive(Debug, Clone, Copy)]
enum ValueType<'ctx> {
    Void,
    Int(IntType<'ctx>),
    Float(FloatType<'ctx>),
    Pointer(PointerType<'ctx>),
    Struct(StructType<'ctx>),
    Array(ArrayType<'ctx>),
}

#[derive(Debug)]
pub struct LLVMGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    machine: TargetMachine,
    builder: inkwell::builder::Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> LLVMGenerator<'ctx> {
    pub fn new(context: &'ctx Context, machine: TargetMachine) -> LLVMGenerator<'ctx> {
        let module = context.create_module("nagi");
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
            machine,
            variables: HashMap::new(),
        }
    }

    pub fn host_machine() -> anyhow::Result<TargetMachine> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| anyhow!("failed to initialize native target: {}", e))?;

        let triple = TargetMachine::get_default_triple();
        let target =
            Target::from_triple(&triple).map_err(|e| anyhow!("failed to create target: {}", e))?;

        let cpu = TargetMachine::get_host_cpu_name();
        let features = TargetMachine::get_host_cpu_features();

        let opt_level = OptimizationLevel::Aggressive;
        let reloc_mode = RelocMode::Default;
        let code_model = CodeModel::Default;

        target
            .create_target_machine(
                &triple,
                cpu.to_str()?,
                features.to_str()?,
                opt_level,
                reloc_mode,
                code_model,
            )
            .ok_or(anyhow!("failed to create target machine"))
    }

    pub fn compile(&mut self, ast: &ASTNode) {
        self.compile_ast_to_llvm(ast);
    }

    pub fn output_o_file(&self, path: &str) -> Result<()> {
        fs::create_dir_all(path).unwrap();
        fs::create_dir_all(&format!("{}/ll", path)).unwrap();

        let module_name = self.module.get_name().to_str()?;
        let file_path = format!("{}{}.o", path, module_name);
        let file = Path::new(&file_path);
        self.module
            .verify()
            .map_err(|e| anyhow!("module verification failed: {}", e))?;
        self.machine
            .write_to_file(&self.module, FileType::Object, file)
            .map_err(|e| anyhow!("failed to write object file: {}", e))?;

        let ll_name = format!("{}/ll/{}.ll", path, module_name);
        let mut ir = std::fs::File::create(ll_name).unwrap();
        ir.write_all(self.module.to_string().as_bytes())?;

        Ok(())
    }

    pub fn output_execute_file(&self, path: &str) -> Result<()> {
        let module_name = self.module.get_name().to_str()?;
        let obj_file_path = format!("{}{}.o", path, module_name);

        self.output_o_file(path)?;

        let clang_status = Command::new("clang")
            .arg("-o")
            .arg("nagi")
            .arg(&obj_file_path)
            .status();

        if clang_status.is_err() {
            println!("compile failed");
        }

        Ok(())
    }

    pub fn print_log(&self) {
        println!("{}", self.module.print_to_string().to_string());
    }

    fn compile_ast_to_llvm(&mut self, ast: &ASTNode) -> Option<Value<'ctx>> {
        let v = match &ast.node_kind {
            ASTNodeKind::Factor { token } => self.compile_factor(token),
            ASTNodeKind::Literal { literal } => self.compile_literal(literal),
            ASTNodeKind::BinaryOperator {
                operator,
                left,
                right,
            } => self.compile_operator(operator, left, right),

            ASTNodeKind::Statement { statement } => {
                self.compile_ast_to_llvm(statement.as_ref().unwrap())?
            }

            ASTNodeKind::Expression { expression } => self.compile_ast_to_llvm(expression)?,
            ASTNodeKind::ExpressionWithoutBlock {
                outer_attribute,
                expression,
            } => self.compile_ast_to_llvm(expression)?,
            ASTNodeKind::ExpressionWithBlock {
                outer_attribute,
                expression_with_block,
            } => self.compile_ast_to_llvm(expression_with_block)?,

            ASTNodeKind::Statements { statements } => {
                for statement in statements {
                    self.compile_ast_to_llvm(statement)?;
                }
                return None;
            }

            ASTNodeKind::IfExpression {
                expression,
                block_expression,
                else_expression,
            } => {
                if self
                    .compile_if_statement(expression, block_expression, else_expression.as_deref())
                    .is_err()
                {
                    panic!();
                }
                return None;
            }

            ASTNodeKind::LoopExpression {
                loop_label,
                loop_expression,
            } => {
                if self.compile_loop(loop_expression).is_err() {
                    panic!();
                }
                return None;
            }

            ASTNodeKind::Crate {
                inner_attribute,
                item,
            } => {
                for attribute in inner_attribute {
                    //
                }

                for i in item {
                    self.compile_ast_to_llvm(i)?;
                }
                return None;
            }

            ASTNodeKind::VisItem { visibility, item } => self.compile_ast_to_llvm(item)?,
            ASTNodeKind::Function {
                function_qualifiers,
                identifier,
                generic_params,
                function_parameters,
                function_return_type,
                where_clause,
                block_expression,
            } => {
                if self
                    .compile_function(
                        function_qualifiers,
                        identifier,
                        generic_params.as_deref(),
                        function_parameters.as_deref(),
                        function_return_type.as_deref(),
                        where_clause.as_deref(),
                        block_expression.as_deref(),
                    )
                    .is_err()
                {
                    panic!();
                }
                return None;
            }

            ASTNodeKind::BlockExpression {
                inner_attribute: _,
                statements,
            } => {
                let Some(statement) = statements else {
                    return None;
                };
                self.compile_ast_to_llvm(statement)?
            }

            ASTNodeKind::LetStatement {
                outer_attribute: _,
                rarity,
                pattern_no_top_alt,
                type_expression,
                expression,
                block_expression,
            } => self.compile_let_statement(
                rarity,
                pattern_no_top_alt,
                type_expression.as_deref(),
                expression.as_deref(),
                block_expression.as_deref(),
            ),

            ASTNodeKind::PathExpression { expression } => self.compile_ast_to_llvm(expression)?,
            ASTNodeKind::PathInExpression { path_expr_segment } => {
                self.compile_ast_to_llvm(path_expr_segment.last().unwrap())?
            }
            ASTNodeKind::PathExprSegment {
                path_ident_segment,
                generic_args: _,
            } => self.compile_ast_to_llvm(path_ident_segment)?,

            _ => panic!("{:?}", ast),
        };

        Some(v)
    }

    fn compile_factor(&self, token: &Token) -> Value<'ctx> {
        match token {
            Token::Identifier(identifier) => {
                let Some(variable) = self.variables.get(identifier) else {
                    panic!();
                };
                Value::Pointer(variable.clone())
            }
            Token::Literal(literal) => self.compile_literal(literal),

            _ => panic!(),
        }
    }

    fn compile_literal(&self, literal: &Literal) -> Value<'ctx> {
        match literal.literal_kind {
            LiteralKind::Integer => Value::Int(self.compile_integer(literal)),
            LiteralKind::Float => Value::Float(self.compile_float(literal)),
            LiteralKind::Bool(boolean) => {
                if boolean {
                    Value::Int(self.context.i8_type().const_int(1, false))
                } else {
                    Value::Int(self.context.i8_type().const_int(0, false))
                }
            }

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
        &mut self,
        operator: &BinaryOperator,
        left: &ASTNode,
        right: &ASTNode,
    ) -> Value<'ctx> {
        let Some(left_value) = self.compile_ast_to_llvm(left) else {
            panic!("left is None: {:?}", left);
        };
        let Some(right_value) = self.compile_ast_to_llvm(right) else {
            panic!("right is None: {:?}", right);
        };

        match (left_value, right_value) {
            (Value::Int(left), Value::Int(right)) => {
                self.compile_operator_int_int(operator, left, right)
            }
            (Value::Float(left), Value::Float(right)) => {
                self.compile_operator_float_float(operator, left, right)
            }
            (Value::Bool(left), Value::Bool(right)) => {
                self.compile_operator_bool_bool(operator, left, right)
            }
            (Value::Pointer(left), _) => {
                self.compile_operator_pointer_any(operator, left, right_value)
            }
            (_, Value::Pointer(right)) => {
                self.compile_operator_any_pointer(operator, left_value, right)
            }
            _ => panic!(
                "\nop: {:?}\nleft: {:?}\nright: {:?}",
                operator, left_value, right_value
            ),
        }
    }

    fn compile_operator_int_int(
        &self,
        operator: &BinaryOperator,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
    ) -> Value<'ctx> {
        let result = match operator {
            BinaryOperator::Add => self.builder.build_int_add(left, right, "tmp_add"),
            BinaryOperator::Sub => self.builder.build_int_sub(left, right, "tmp_sub"),
            BinaryOperator::Mul => self.builder.build_int_mul(left, right, "tmp_mul"),
            BinaryOperator::Div => self.builder.build_int_signed_div(left, right, "tmp_div"),
            BinaryOperator::And => self.builder.build_and(left, right, "tmp_and"),
            BinaryOperator::Or => self.builder.build_or(left, right, "tmp_or"),
            BinaryOperator::Xor => self.builder.build_xor(left, right, "tmp_xor"),
            _ => {
                return self.compile_condition_operator(
                    operator,
                    Value::Int(left),
                    Value::Int(right),
                )
            }
        }
        .unwrap();

        Value::Int(result)
    }

    fn compile_condition_and(&self, left: IntValue<'ctx>, right: IntValue<'ctx>) -> IntValue<'ctx> {
        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        let then_block = self.context.append_basic_block(function, "condition_then");
        let else_block = self.context.append_basic_block(function, "condition_else");
        let merge_block = self.context.append_basic_block(function, "condition_merge");

        self.builder
            .build_conditional_branch(left, then_block, else_block)
            .unwrap();

        self.builder.position_at_end(then_block);
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();
        let right_value = right;

        self.builder.position_at_end(else_block);
        let false_value = self.context.bool_type().const_int(0, false);
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();

        self.builder.position_at_end(merge_block);
        let phi = self
            .builder
            .build_phi(self.context.bool_type(), "phi")
            .unwrap();
        phi.add_incoming(&[(&right_value, then_block), (&false_value, else_block)]);

        phi.as_basic_value().into_int_value()
    }

    fn compile_condition_or(&self, left: IntValue<'ctx>, right: IntValue<'ctx>) -> IntValue<'ctx> {
        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        let then_block = self.context.append_basic_block(function, "then");
        let else_block = self.context.append_basic_block(function, "else");
        let merge_block = self.context.append_basic_block(function, "merge");

        self.builder
            .build_conditional_branch(left, then_block, else_block)
            .unwrap();

        self.builder.position_at_end(then_block);
        let true_value = self.context.bool_type().const_int(1, false);
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();

        self.builder.position_at_end(else_block);
        let right_value = right;
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();

        self.builder.position_at_end(merge_block);
        let phi = self
            .builder
            .build_phi(self.context.bool_type(), "phi")
            .unwrap();
        phi.add_incoming(&[(&true_value, then_block), (&right_value, else_block)]);

        phi.as_basic_value().into_int_value()
    }

    fn compile_operator_float_float(
        &self,
        operator: &BinaryOperator,
        left: FloatValue<'ctx>,
        right: FloatValue<'ctx>,
    ) -> Value<'ctx> {
        let result = match operator {
            BinaryOperator::Add => self.builder.build_float_add(left, right, "addtmp"),
            BinaryOperator::Sub => self.builder.build_float_sub(left, right, "subtmp"),
            BinaryOperator::Mul => self.builder.build_float_mul(left, right, "multmp"),
            BinaryOperator::Div => self.builder.build_float_div(left, right, "divtmp"),
            _ => {
                return self.compile_condition_operator(
                    operator,
                    Value::Float(left),
                    Value::Float(right),
                )
            }
        }
        .unwrap();
        Value::Float(result)
    }

    fn compile_operator_pointer_any(
        &self,
        operator: &BinaryOperator,
        left: PointerValue<'ctx>,
        right: Value<'ctx>,
    ) -> Value<'ctx> {
        match operator {
            BinaryOperator::Equal => self.compile_assign_operator(None, left, right),
            BinaryOperator::AddAssign => {
                self.compile_assign_operator(Some(&BinaryOperator::Add), left, right)
            }
            BinaryOperator::SubAssign => {
                self.compile_assign_operator(Some(&BinaryOperator::Sub), left, right)
            }
            BinaryOperator::MulAssign => {
                self.compile_assign_operator(Some(&BinaryOperator::Mul), left, right)
            }
            BinaryOperator::DivAssign => {
                self.compile_assign_operator(Some(&BinaryOperator::Div), left, right)
            }
            BinaryOperator::ModAssign => {
                self.compile_assign_operator(Some(&BinaryOperator::Mod), left, right)
            }
            BinaryOperator::XorAssign => {
                self.compile_assign_operator(Some(&BinaryOperator::Xor), left, right)
            }
            BinaryOperator::OrAssign => {
                self.compile_assign_operator(Some(&BinaryOperator::Or), left, right)
            }
            BinaryOperator::AndAssign => {
                self.compile_assign_operator(Some(&BinaryOperator::And), left, right)
            }
            _ => {
                let load_variable = self.builder.build_load(left, "load_variable").unwrap();

                match (load_variable, right) {
                    (BasicValueEnum::IntValue(left), Value::Int(right)) => {
                        self.compile_operator_int_int(operator, left, right)
                    }
                    (BasicValueEnum::FloatValue(left), Value::Float(right)) => {
                        self.compile_operator_float_float(operator, left, right)
                    }
                    (_, Value::Pointer(right)) => {
                        let left = load_variable.as_basic_value_enum();
                        let right = self.builder.build_load(right, "load_variable").unwrap();
                        self.compile_operator_any_any(operator, left, right)
                    }
                    _ => panic!("\n{:?}, \n{:?}", left, right),
                }
            }
        }
    }

    fn compile_operator_any_pointer(
        &self,
        operator: &BinaryOperator,
        left: Value<'ctx>,
        right: PointerValue<'ctx>,
    ) -> Value<'ctx> {
        let load_variable = self.builder.build_load(right, "load_variable").unwrap();

        match (left, load_variable) {
            (Value::Int(left), BasicValueEnum::IntValue(right)) => {
                self.compile_operator_int_int(operator, left, right)
            }
            (Value::Float(left), BasicValueEnum::FloatValue(right)) => {
                self.compile_operator_float_float(operator, left, right)
            }
            (Value::Pointer(left), _) => {
                let left = self.builder.build_load(left, "load_variable").unwrap();
                let right = load_variable.as_basic_value_enum();
                self.compile_operator_any_any(operator, left, right)
            }

            _ => panic!(),
        }
    }

    fn compile_operator_any_any(
        &self,
        operator: &BinaryOperator,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Value<'ctx> {
        match (left, right) {
            (BasicValueEnum::IntValue(left), BasicValueEnum::IntValue(right)) => {
                self.compile_operator_int_int(operator, left, right)
            }
            (BasicValueEnum::FloatValue(left), BasicValueEnum::FloatValue(right)) => {
                self.compile_operator_float_float(operator, left, right)
            }

            _ => panic!("{:?}, {:?}", left, right),
        }
    }

    fn compile_operator_bool_bool(
        &self,
        operator: &BinaryOperator,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
    ) -> Value<'ctx> {
        self.compile_condition_operator(operator, Value::Int(left), Value::Int(right))
    }

    fn compile_condition_operator(
        &self,
        operator: &BinaryOperator,
        left: Value<'ctx>,
        right: Value<'ctx>,
    ) -> Value<'ctx> {
        match (left, right) {
            (Value::Int(left), Value::Int(right)) => {
                let result = match operator {
                    BinaryOperator::ConditionCompare => {
                        self.builder
                            .build_int_compare(IntPredicate::EQ, left, right, "tmp_cmp_eq")
                    }
                    BinaryOperator::ConditionNotCompare => {
                        self.builder
                            .build_int_compare(IntPredicate::NE, left, right, "tmp_cmp_ne")
                    }
                    BinaryOperator::ConditionLessThan => self.builder.build_int_compare(
                        IntPredicate::SLT,
                        left,
                        right,
                        "tmp_cmp_slt",
                    ),
                    BinaryOperator::ConditionLessThanEquel => self.builder.build_int_compare(
                        IntPredicate::SLE,
                        left,
                        right,
                        "tmp_cmp_sle",
                    ),
                    BinaryOperator::ConditionGreaterThan => self.builder.build_int_compare(
                        IntPredicate::SGT,
                        left,
                        right,
                        "tmp_cmp_sgt",
                    ),
                    BinaryOperator::ConditionGreaterThanEqual => self.builder.build_int_compare(
                        IntPredicate::SGE,
                        left,
                        right,
                        "tmp_cmp_sge",
                    ),
                    BinaryOperator::ConditionAnd => {
                        return Value::Bool(self.compile_condition_and(left, right))
                    }
                    BinaryOperator::ConditionOr => {
                        return Value::Bool(self.compile_condition_or(left, right))
                    }
                    _ => panic!("{:?}", operator),
                }
                .unwrap();

                Value::Bool(result)
            }
            (Value::Float(left), Value::Float(right)) => {
                let result =
                    match operator {
                        BinaryOperator::ConditionLessThan => self.builder.build_float_compare(
                            FloatPredicate::OLT,
                            left,
                            right,
                            "tmp_cmp_olt",
                        ),
                        BinaryOperator::ConditionLessThanEquel => self.builder.build_float_compare(
                            FloatPredicate::OLE,
                            left,
                            right,
                            "tmp_cmp_ole",
                        ),
                        BinaryOperator::ConditionGreaterThan => self.builder.build_float_compare(
                            FloatPredicate::OGT,
                            left,
                            right,
                            "tmp_cmp_ogt",
                        ),
                        BinaryOperator::ConditionGreaterThanEqual => self
                            .builder
                            .build_float_compare(FloatPredicate::OGE, left, right, "tmp_cmp_oge"),
                        _ => panic!(),
                    }
                    .unwrap();

                Value::Bool(result)
            }
            _ => panic!(),
        }
    }

    fn compile_assign_operator(
        &self,
        sub_operator: Option<&BinaryOperator>,
        left: PointerValue<'ctx>,
        right: Value<'ctx>,
    ) -> Value<'ctx> {
        let result = if let Some(sub_operator) = sub_operator {
            self.compile_operator_pointer_any(sub_operator, left, right)
        } else {
            right
        };

        match result {
            Value::Bool(result) => self.builder.build_store(left, result),
            Value::Int(result) => self.builder.build_store(left, result),
            Value::Float(result) => self.builder.build_store(left, result),
            Value::Pointer(result) => self.builder.build_store(left, result),
            Value::Struct(result) => self.builder.build_store(left, result),
            Value::Array(result) => self.builder.build_store(left, result),
        }
        .unwrap();

        Value::Pointer(left)
    }

    fn compile_let_statement(
        &mut self,
        rarity: &Rarity,
        pattern_no_top_alt: &ASTNode,
        type_expression: Option<&ASTNode>,
        expression: Option<&ASTNode>,
        block_expression: Option<&ASTNode>,
    ) -> Value<'ctx> {
        let ASTNodeKind::PatternNoTopAlt { pattern } = &pattern_no_top_alt.node_kind else {
            panic!();
        };
        let ASTNodeKind::PatternWithoutRange { pattern } = &pattern.node_kind else {
            panic!();
        };

        let variable_name = match &pattern.node_kind {
            ASTNodeKind::IdentifierPattern {
                ref_keyword: _,
                mut_keyword: _,
                identifier,
                pattern_no_top_alt: _,
            } => identifier.as_str(),
            _ => "temp",
        };

        // TODO
        let i32_type = self.context.i32_type();
        let init_value = i32_type.const_int(0, false);

        let Some(let_expr) = expression else {
            let temp_value = self.builder.build_alloca(i32_type, variable_name).unwrap();
            if self.builder.build_store(temp_value, init_value).is_err() {
                panic!();
            }
            return Value::Pointer(temp_value);
        };

        let Some(value) = self.compile_ast_to_llvm(let_expr) else {
            let temp_value = self.builder.build_alloca(i32_type, variable_name).unwrap();
            if self.builder.build_store(temp_value, init_value).is_err() {
                panic!();
            }
            return Value::Pointer(temp_value);
        };

        let variable = match value {
            Value::Int(int_value) => {
                let local_value = self
                    .builder
                    .build_alloca(int_value.get_type(), variable_name)
                    .unwrap();
                self.builder.build_store(local_value, int_value);
                local_value
            }
            Value::Float(float_value) => {
                let local_value = self
                    .builder
                    .build_alloca(float_value.get_type(), variable_name)
                    .unwrap();
                self.builder.build_store(local_value, float_value);
                local_value
            }
            Value::Pointer(pointer_value) => {
                let local_value = self
                    .builder
                    .build_alloca(pointer_value.get_type(), variable_name)
                    .unwrap();
                self.builder
                    .build_store(local_value, pointer_value)
                    .unwrap();
                local_value
            }

            _ => panic!(),
        };

        self.variables.insert(variable_name.to_string(), variable);

        Value::Pointer(variable)
    }

    fn compile_if_statement(
        &mut self,
        expression: &ASTNode,
        block_expression: &ASTNode,
        else_expression: Option<&ASTNode>,
    ) -> Result<()> {
        // 条件式が最終的にBoolを返さなければエラー
        let Some(condition) = self.compile_ast_to_llvm(expression) else {
            panic!();
        };
        let Value::Bool(condition_value) = condition else {
            panic!();
        };

        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        let then_block = self.context.append_basic_block(function, "if_then_block");
        let merge_block = self.context.append_basic_block(function, "if_end_block");

        // else ブロック判定
        if let Some(else_expr) = else_expression {
            let else_block = self.context.append_basic_block(function, "if_else_block");
            self.builder
                .build_conditional_branch(condition_value, then_block, else_block)?;

            // then
            self.builder.position_at_end(then_block);
            self.compile_ast_to_llvm(block_expression);
            self.builder.build_unconditional_branch(merge_block)?;

            // else
            self.builder.position_at_end(else_block);
            self.compile_ast_to_llvm(else_expr);
            self.builder.build_unconditional_branch(merge_block)?;
        } else {
            self.builder
                .build_conditional_branch(condition_value, then_block, merge_block)?;

            // then
            self.builder.position_at_end(then_block);
            self.compile_ast_to_llvm(block_expression);
            self.builder.build_unconditional_branch(merge_block)?;
        }

        // end
        self.builder.position_at_end(merge_block);

        Ok(())
    }

    fn compile_loop(&mut self, loop_expression: &ASTNode) -> Result<()> {
        match &loop_expression.node_kind {
            ASTNodeKind::InfiniteLoopExpression { block_expression } => {
                self.compile_infinite_loop(block_expression)
            }
            ASTNodeKind::PredicateLoopExpression {
                expression,
                block_expression,
            } => self.compile_predicate_loop(expression, block_expression),
            _ => panic!(),
        }
    }

    fn compile_infinite_loop(&mut self, block_expression: &ASTNode) -> Result<()> {
        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        let loop_entry = self.context.append_basic_block(function, "loop_entry");
        let loop_block = self.context.append_basic_block(function, "loop_block");

        self.builder.build_unconditional_branch(loop_entry)?;

        self.builder.position_at_end(loop_entry);
        self.builder.build_unconditional_branch(loop_block)?;

        self.builder.position_at_end(loop_block);

        self.compile_ast_to_llvm(block_expression);

        self.builder.build_unconditional_branch(loop_block)?;

        Ok(())
    }

    fn compile_predicate_loop(
        &mut self,
        expression: &ASTNode,
        block_expression: &ASTNode,
    ) -> Result<()> {
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
        self.builder.build_unconditional_branch(condition_block)?;

        self.builder.position_at_end(condition_block);
        let Some(condition_value) = self.compile_ast_to_llvm(expression) else {
            panic!();
        };
        let Value::Bool(condition) = condition_value else {
            panic!("Condition must return an integer");
        };

        self.builder
            .build_conditional_branch(condition, body_block, end_block)?;

        self.builder.position_at_end(body_block);
        self.compile_ast_to_llvm(block_expression);
        self.builder.build_unconditional_branch(condition_block)?;

        self.builder.position_at_end(end_block);

        Ok(())
    }

    fn compile_function(
        &mut self,
        function_qualifiers: &ASTNode,
        identifier: &str,
        generic_params: Option<&ASTNode>,
        function_parameters: Option<&ASTNode>,
        function_return_type: Option<&ASTNode>,
        where_clause: Option<&ASTNode>,
        block_expression: Option<&ASTNode>,
    ) -> Result<()> {
        let params_type = if let Some(parameters) = function_parameters {
            self.get_function_parameters(parameters)
        } else {
            vec![]
        };

        let funtion_type = if let Some(return_type) = function_return_type {
            match self.get_type(return_type) {
                ValueType::Void => self.context.void_type().fn_type(&params_type, false),
                ValueType::Int(int_type) => int_type.fn_type(&params_type, false),
                ValueType::Float(float_type) => float_type.fn_type(&params_type, false),
                _ => panic!(),
            }
        } else {
            self.context.void_type().fn_type(&params_type, false)
        };

        let function = self.module.add_function(identifier, funtion_type, None);
        let block = self.context.append_basic_block(function, "block");
        self.builder.position_at_end(block);

        let Some(expression) = block_expression else {
            panic!(); // TODO
        };

        // ブロック内
        let Some(return_value) = self.compile_ast_to_llvm(expression) else {
            self.builder.build_return(None)?;
            return Ok(());
        };

        match return_value {
            Value::Int(int_value) => self.builder.build_return(Some(&int_value))?,
            Value::Float(float_value) => self.builder.build_return(Some(&float_value))?,
            Value::Pointer(pointer_value) => self.builder.build_return(Some(&pointer_value))?,
            _ => self.builder.build_return(None)?,
        };

        Ok(())
    }

    fn get_function_parameters(
        &self,
        function_params: &ASTNode,
    ) -> Vec<BasicMetadataTypeEnum<'ctx>> {
        let ASTNodeKind::FunctionParameters {
            self_param,
            function_param,
        } = &function_params.node_kind
        else {
            panic!();
        };

        let mut vec = vec![];
        for param in function_param {
            let ASTNodeKind::FunctionParamPattern {
                pattern_no_top_alt,
                type_expression,
            } = &param.node_kind
            else {
                panic!();
            };

            let _type = match self.get_type(&type_expression) {
                ValueType::Int(int_type) => int_type.into(),
                ValueType::Float(float_type) => float_type.into(),
                _ => panic!(),
            };

            vec.push(_type);
        }

        vec
    }

    fn get_type(&self, type_expression: &ASTNode) -> ValueType<'ctx> {
        let ASTNodeKind::Type { types } = &type_expression.node_kind else {
            panic!();
        };

        match &types {
            Types::Integer {
                bit_width,
                is_unsigned,
            } => {
                let integer = match bit_width {
                    BitWidth::Bit8 => self.context.i8_type(),
                    BitWidth::Bit16 => self.context.i16_type(),
                    BitWidth::Bit32 => self.context.i32_type(),
                    BitWidth::Bit64 => self.context.i64_type(),
                    BitWidth::Bit128 => self.context.i128_type(),
                };

                ValueType::Int(integer)
            }
            Types::Float(_) => ValueType::Void,
            Types::Pointer => ValueType::Void,
            Types::Array => ValueType::Void,
            Types::Struct(_) => ValueType::Void,

            _ => panic!(),
        }
    }
}
