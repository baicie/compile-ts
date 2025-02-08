mod codegen;
mod module_resolver;
mod symbol_table;
mod type_checker;
mod types;

use codegen::CodeGenerator;
use module_resolver::ModuleResolver;
use type_checker::TypeChecker;

use clap::{Arg, Command};
use inkwell::context::Context;
use std::path::PathBuf;
use std::process::{self};
use std::time::Instant;

#[derive(Debug)]
pub struct CompilerOptions {
    pub target: Option<String>,
    pub input_file: String,
    pub output_file: String,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            target: Some(std::env::consts::ARCH.to_string()),
            input_file: String::new(),
            output_file: String::new(),
        }
    }
}

pub fn run_compiler(options: CompilerOptions) -> i32 {
    let entry_point = PathBuf::from(&options.input_file);
    let mut resolver = ModuleResolver::new(entry_point);

    // 解析所有相关模块
    if let Err(e) = resolver.resolve_all() {
        println!("Module resolution error: {}", e);
        return 1;
    }

    // 对所有模块进行类型检查
    let mut type_checker = TypeChecker::new();
    for (path, program) in resolver.get_all_modules() {
        match type_checker.check(program) {
            Ok(_) => println!("Type checking passed for {}", path.display()),
            Err(e) => {
                println!("Type error in {}: {}", path.display(), e);
                return 1;
            }
        }
    }

    // 生成代码
    let context = Context::create();
    let mut code_generator = CodeGenerator::new(&context, "output");

    // 为每个模块生成代码
    for (_, program) in resolver.get_all_modules() {
        if let Err(e) = code_generator.generate(program) {
            println!("Code generation error: {}", e);
            return 1;
        }
    }

    // 优化
    code_generator.optimize();
    println!("Optimization completed");

    // 输出 IR (用于调试)
    code_generator.print_to_file("output.ll");

    // 生成目标文件
    match code_generator.generate_object_file("output.o", options.target) {
        Ok(_) => println!("Object file generated successfully"),
        Err(e) => {
            println!("Failed to generate object file: {}", e);
            return 1;
        }
    }

    0
}

pub fn main() {
    let _start_time = Instant::now();

    let matches = Command::new("tsc")
        .version("1.0")
        .about("TypeScript compiler in Rust")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .help("Input file")
                .default_value("index.ts"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file")
                .default_value("out.o"),
        )
        .arg(
            Arg::new("target")
                .short('t')
                .long("target")
                .help("Target architecture"),
        )
        .get_matches();

    let exit_code = run_compiler(CompilerOptions {
        target: matches.get_one::<String>("target").cloned(),
        input_file: matches.get_one::<String>("input").unwrap().clone(),
        output_file: matches.get_one::<String>("output").unwrap().clone(),
    });

    process::exit(exit_code)
}
