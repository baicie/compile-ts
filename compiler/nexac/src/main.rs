//! Nexa 编译器主程序
//!
//! 读取源文件，解析为 AST，生成 LLVM IR，编译为可执行文件。

use inkwell::context::Context;
use inkwell::targets::Target;
use inkwell::OptimizationLevel;
use nexa_codegen::CodeGenerator;
use nexa_parser::{Parser, Program};
use std::process::Command;

/// 打印解析错误，包含源码位置上下文
fn print_parse_error(source: &str, error: &nexa_parser::ParseError) {
    let span = &error.span;
    eprintln!("error: {}", error.message);
    eprintln!("  --> {}:{}:{}", "source", span.start.0 + 1, span.start.1 + 1);

    // 显示错误行的上下文
    let lines: Vec<&str> = source.lines().collect();
    if span.start.0 < lines.len() {
        let line = lines[span.start.0];
        eprintln!("   |");
        eprintln!("{} | {}", span.start.0 + 1, line);

        // 显示错误位置指示
        let col = span.start.1;
        let indicator = format!("{:>width$}^", "", width = col + 4);
        eprintln!("   | {}", indicator);
    }
}

/// CLI 配置
#[derive(Debug)]
struct CliOptions {
    source_file: String,
    output_file: Option<String>,
    opt_level: OptimizationLevel,
    target_triple: Option<String>,
    output_type: OutputType,
    debug_ast: bool,
    #[allow(dead_code)]
    emit_llvm: bool,
    #[allow(dead_code)]
    emit_asm: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OutputType {
    Exe,
    LLVMIR,
    Asm,
    Object,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return Ok(());
    }

    // 解析 CLI 参数
    let cli = parse_args(&args)?;

    println!("Nexa compiler v0.1.0");
    println!("Compiling: {}", cli.source_file);

    // 读取源文件
    let source = std::fs::read_to_string(&cli.source_file)?;

    // 解析源文件
    let mut parser = Parser::new(&source);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            print_parse_error(&source, &e);
            return Err(format!("Parse error: {}", e.message).into());
        },
    };

    println!("Parsed {} functions", program.functions.len());

    // 调试：打印 AST
    if cli.debug_ast {
        println!("\n=== AST ===");
        for func in &program.functions {
            println!("Function: {}", func.name);
            println!("  Body: {:?}", func.body);
        }
    }

    // 编译并执行
    compile_and_execute(&program, &cli)?;

    Ok(())
}

/// 打印使用说明
fn print_usage(program_name: &str) {
    println!("Usage: {} <source_file> [options]", program_name);
    println!("Compile Nexa/TypeScript source file to executable");
    println!("\nOptions:");
    println!("  -o, --output <file>    Output file name");
    println!("  --opt-level <level>    Optimization level: 0, 1, 2, 3, s, z (default: 0)");
    println!("  --target <triple>      Target triple (e.g., x86_64-linux-gnu)");
    println!("  --output-type <type>   Output type: exe, llvm-ir, asm, obj (default: exe)");
    println!("  -d, --debug            Print debug information (AST)");
    println!("  --emit-llvm            Emit LLVM IR");
    println!("  --emit-asm             Emit assembly");
    println!("  -h, --help             Show this help message");
    println!("\nExamples:");
    println!("  {} hello.nexa                     # Compile and show IR", program_name);
    println!("  {} hello.nexa -o hello            # Compile to executable", program_name);
    println!("  {} hello.nexa --opt-level 3       # Optimize with -O3", program_name);
    println!("  {} hello.nexa --emit-llvm         # Output LLVM IR", program_name);
}

/// 解析命令行参数
fn parse_args(args: &[String]) -> Result<CliOptions, Box<dyn std::error::Error>> {
    let mut source_file = None;
    let mut output_file = None;
    let mut opt_level = OptimizationLevel::None;
    let mut target_triple = None;
    let mut output_type = OutputType::Exe;
    let mut debug_ast = false;
    let mut emit_llvm = false;
    let mut emit_asm = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage(&args[0]);
                std::process::exit(0);
            },
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing output file name".into());
                }
            },
            "--opt-level" => {
                if i + 1 < args.len() {
                    opt_level = match args[i + 1].as_str() {
                        "0" => OptimizationLevel::None,
                        "1" => OptimizationLevel::Less,
                        "2" => OptimizationLevel::Default,
                        "3" => OptimizationLevel::Aggressive,
                        "s" | "z" => OptimizationLevel::Default,
                        _ => return Err(format!("Invalid optimization level: {}", args[i + 1]).into()),
                    };
                    i += 2;
                } else {
                    return Err("Missing optimization level".into());
                }
            },
            "--target" => {
                if i + 1 < args.len() {
                    target_triple = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing target triple".into());
                }
            },
            "--output-type" => {
                if i + 1 < args.len() {
                    output_type = match args[i + 1].as_str() {
                        "exe" => OutputType::Exe,
                        "llvm-ir" => OutputType::LLVMIR,
                        "asm" => OutputType::Asm,
                        "obj" => OutputType::Object,
                        _ => return Err(format!("Invalid output type: {}", args[i + 1]).into()),
                    };
                    i += 2;
                } else {
                    return Err("Missing output type".into());
                }
            },
            "-d" | "--debug" => {
                debug_ast = true;
                i += 1;
            },
            "--emit-llvm" => {
                emit_llvm = true;
                i += 1;
            },
            "--emit-asm" => {
                emit_asm = true;
                i += 1;
            },
            _ => {
                // 认为是源文件
                if args[i].starts_with('-') {
                    return Err(format!("Unknown option: {}", args[i]).into());
                }
                source_file = Some(args[i].clone());
                i += 1;
            },
        }
    }

    let source_file = source_file.ok_or("No source file specified")?;

    // 如果指定了 --emit-llvm 或 --emit-asm，覆盖 output_type
    if emit_llvm {
        output_type = OutputType::LLVMIR;
    }
    if emit_asm {
        output_type = OutputType::Asm;
    }

    Ok(CliOptions {
        source_file,
        output_file,
        opt_level,
        target_triple,
        output_type,
        debug_ast,
        emit_llvm,
        emit_asm,
    })
}

/// 编译并执行代码
fn compile_and_execute(
    program: &Program,
    cli: &CliOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 LLVM 目标
    Target::initialize_native(&Default::default())?;

    // 创建 LLVM 上下文和模块
    let context = Context::create();
    let mut codegen = CodeGenerator::new(&context, "nexa_module");

    // 声明内置函数
    codegen.declare_builtin_functions();

    // 生成程序
    codegen
        .generate_program(program)
        .map_err(|e| format!("Code generation error: {}", e.message))?;

    // 获取生成的模块
    let module = codegen.into_module();

    // 验证模块
    module.verify().map_err(|e| format!("Module verification failed: {}", e))?;

    // 根据 output_type 处理输出
    match cli.output_type {
        OutputType::LLVMIR => {
            println!("\nGenerated LLVM IR:");
            println!("{}", module.print_to_string().to_string_lossy());
        },
        OutputType::Asm => {
            let triple_str = cli.target_triple.clone().unwrap_or_else(host_target_triple);
            let target_triple = inkwell::targets::TargetTriple::create(&triple_str);
            let target = inkwell::targets::Target::from_triple(&target_triple)?;

            let target_machine = target
                .create_target_machine(
                    &target_triple,
                    "generic",
                    "",
                    cli.opt_level,
                    inkwell::targets::RelocMode::Default,
                    inkwell::targets::CodeModel::Default,
                )
                .ok_or("Failed to create target machine")?;

            let output = cli.output_file.clone().unwrap_or_else(|| "a.s".to_string());
            target_machine
                .write_to_file(&module, inkwell::targets::FileType::Assembly, output.as_ref())
                .map_err(|e| format!("Failed to write assembly file: {}", e))?;

            println!("\nCompiled to assembly file: {}", output);
        },
        OutputType::Object => {
            let triple_str = cli.target_triple.clone().unwrap_or_else(host_target_triple);
            let target_triple = inkwell::targets::TargetTriple::create(&triple_str);
            let target = inkwell::targets::Target::from_triple(&target_triple)?;

            let target_machine = target
                .create_target_machine(
                    &target_triple,
                    "generic",
                    "",
                    cli.opt_level,
                    inkwell::targets::RelocMode::Default,
                    inkwell::targets::CodeModel::Default,
                )
                .ok_or("Failed to create target machine")?;

            let output = cli.output_file.clone().ok_or("Output file required for object file")?;
            target_machine
                .write_to_file(&module, inkwell::targets::FileType::Object, output.as_ref())
                .map_err(|e| format!("Failed to write object file: {}", e))?;

            println!("\nCompiled to object file: {}", output);
        },
        OutputType::Exe => {
            println!("\nGenerated LLVM IR:");
            println!("{}", module.print_to_string().to_string_lossy());

            if let Some(output) = &cli.output_file {
                // 生成可执行文件
                let triple_str = cli.target_triple.clone().unwrap_or_else(host_target_triple);
                let target_triple = inkwell::targets::TargetTriple::create(&triple_str);
                let target = inkwell::targets::Target::from_triple(&target_triple)?;

                let target_machine = target
                    .create_target_machine(
                        &target_triple,
                        "generic",
                        "",
                        cli.opt_level,
                        inkwell::targets::RelocMode::Default,
                        inkwell::targets::CodeModel::Default,
                    )
                    .ok_or("Failed to create target machine")?;

                let temp_dir = std::env::temp_dir();
                let object_file = temp_dir.join("nexa_temp.o");

                target_machine
                    .write_to_file(&module, inkwell::targets::FileType::Object, object_file.as_path())
                    .map_err(|e| format!("Failed to write object file: {}", e))?;

                let link_status = Command::new("clang").arg(&object_file).arg("-o").arg(output).status()?;

                let _ = std::fs::remove_file(&object_file);

                if link_status.success() {
                    println!("\nCompiled to executable: {}", output);
                } else {
                    return Err("Linker failed to produce executable".into());
                }
            } else {
                println!("\nNote: Use -o <file> to generate executable file");
            }
        },
    }

    Ok(())
}

/// 返回当前主机对应的 LLVM target triple 字符串
fn host_target_triple() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    let (llvm_arch, vendor, env) = match (arch, os) {
        ("x86_64", "macos") => ("x86_64", "apple", "macosx"),
        ("aarch64", "macos") => ("arm64", "apple", "macosx"),
        ("x86_64", "linux") => ("x86_64", "unknown", "linux-gnu"),
        ("aarch64", "linux") => ("aarch64", "unknown", "linux-gnu"),
        ("x86_64", "windows") => ("x86_64", "pc", "windows-msvc"),
        ("aarch64", "windows") => ("aarch64", "pc", "windows-msvc"),
        _ => ("x86_64", "unknown", "unknown"),
    };

    format!("{llvm_arch}-{vendor}-{env}")
}
