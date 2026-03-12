//! Nexa 编译器主程序
//!
//! 读取源文件，解析为 AST，生成 LLVM IR，编译为可执行文件。

use inkwell::context::Context;
use inkwell::targets::Target;
use nexa_codegen::CodeGenerator;
use nexa_parser::{Parser, Program};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: nexac <source_file> [-o <output_file>]");
        println!("Compile Nexa/TypeScript source file to executable");
        println!("\nExamples:");
        println!("  nexac hello.ts          # Compile and run");
        println!("  nexac hello.ts -o hello # Compile to executable");
        return Ok(());
    }

    let source_file = &args[1];
    let output_file = if let Some(pos) = args.iter().position(|a| a == "-o") {
        args.get(pos + 1).cloned()
    } else {
        None
    };

    println!("Nexa compiler v0.1.0");
    println!("Compiling: {}", source_file);

    // 读取源文件
    let source = std::fs::read_to_string(source_file)?;

    // 解析源文件
    let mut parser = Parser::new(&source);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {} at {:?}", e.message, e.span);
            return Err(format!("Parse error: {}", e.message).into());
        },
    };

    println!("Parsed {} functions", program.functions.len());

    // 编译并执行
    compile_and_execute(&program, output_file.as_deref())?;

    Ok(())
}

/// 编译并执行代码
fn compile_and_execute(
    program: &Program,
    output_file: Option<&str>,
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

    println!("\nGenerated LLVM IR:");
    println!("{}", module.print_to_string().to_string_lossy());

    if let Some(output) = output_file {
        // 生成可执行文件
        let triple_str = host_target_triple();
        let target_triple = inkwell::targets::TargetTriple::create(&triple_str);
        let target = inkwell::targets::Target::from_triple(&target_triple)?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                inkwell::OptimizationLevel::None,
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
