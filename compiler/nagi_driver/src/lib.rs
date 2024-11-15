use std::{env, fs, process, time::Instant};

use nagi_command_option::CompileCommandOption;

#[derive(Debug)]
pub enum ExitStatus {
    Success = 0,
    CompileFailure = -1,
    UnknownCommand = -2,
    InvalidArgs = -3,
}

pub fn driver() {
    let start_time = Instant::now();
    //let pragram: String = env::args().next().unwrap();
    let Some(command) = env::args().nth(1) else {
        process::exit(ExitStatus::UnknownCommand as i32);
    };
    let args: Vec<String> = env::args().skip(2).collect();

    let result = match command.as_str() {
        "compile" => run_compiler(&args),
        _ => ExitStatus::UnknownCommand,
    };

    println!(
        "exit {:?}, time {}ms",
        result,
        start_time.elapsed().as_millis()
    );

    process::exit(result as i32);
}

fn run_compiler(args: &Vec<String>) -> ExitStatus {
    println!("workdir : {}", env::current_dir().unwrap().display());
    let Ok(compile_option) = CompileCommandOption::new(args) else {
        return ExitStatus::InvalidArgs;
    };

    let mut cst_list = vec![];
    for target in compile_option.target_list.iter() {
        let Ok(code) = open_file(target) else {
            return ExitStatus::CompileFailure;
        };

        let Ok(cst) = nagi_parse::parse(&code, &compile_option) else {
            return ExitStatus::CompileFailure;
        };

        cst_list.push(cst);
    }

    let mut ast_list = vec![];
    for cst in cst_list.iter() {
        let Ok(ast) = nagi_checker::check(cst) else {
            return ExitStatus::CompileFailure;
        };

        ast.write_ast("a.json"); // test
        ast_list.push(ast);
    }

    ExitStatus::Success
}

fn open_file(file_path: &str) -> Result<String, ()> {
    let Ok(sorce_code) = fs::read_to_string(file_path) else {
        return Err(()); //TODO
    };

    Ok(sorce_code)
}
