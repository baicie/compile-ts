use clap::{Arg, Command};
use inkwell::context::Context;
use std::fs;
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
    let source_code = fs::read_to_string(&options.input_file).expect("Failed to read input file");

    // 词法分析
    let lexer = Lexer::new(source_code);
    let mut parser = Parser::new(lexer);

    // 语法分析
    let ast = parser.parse_program();

    // 类型检查
    let mut type_checker = TypeChecker::new();
    match type_checker.check(&ast) {
        Ok(_) => println!("Type checking passed"),
        Err(e) => {
            println!("Type error: {}", e);
            return 1;
        }
    }

    // 生成 LLVM IR
    let context = Context::create();
    let mut code_generator = CodeGenerator::new(&context, "output");
    match code_generator.generate(&ast) {
        Ok(_) => {
            println!("LLVM IR generated successfully");

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
        }
        Err(e) => {
            println!("Code generation error: {}", e);
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

pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod symbol_table;
pub mod type_checker;
pub mod types;

use codegen::CodeGenerator;
pub use lexer::{Lexer, Token, TokenKind};
use parser::Parser;
use type_checker::TypeChecker;
