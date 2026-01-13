mod llvm_ir;

use crate::llvm_ir::llvm_ir_generator::LLVMGenerator;
use inkwell::context::Context;
use nagi_syntax_tree::ast::*;

pub fn ir_generator(ast: &ASTNode) {
    let context = Context::create();
    let machine = LLVMGenerator::host_machine().expect("failed to create machine");
    let mut llvm_generator = LLVMGenerator::new(&context, machine);

    llvm_generator.compile(ast);

    if let Err(e) = llvm_generator.output_execute_file("./test") {
        llvm_generator.print_log();
        panic!("{:?}", e);
    }
}
