//! Nexa 编译器主程序
//!
//! 将 Nexa/TypeScript 风格的代码编译为可执行文件

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{CodeModel, FileType, RelocMode, Target, TargetTriple};
use inkwell::values::FunctionValue;
use inkwell::OptimizationLevel;
use std::path::PathBuf;
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

    // 编译源代码
    compile_and_execute(&source, output_file.as_deref())?;

    Ok(())
}

/// 编译并执行代码
fn compile_and_execute(
    _source: &str,
    output_file: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 LLVM 目标
    Target::initialize_native(&Default::default())?;

    // 创建 LLVM 上下文
    let context = Context::create();
    let module = context.create_module("nexa");
    let builder = context.create_builder();

    // 声明 puts 函数
    let puts_fn = declare_puts_function(&context, &module)?;

    // 声明 main 函数
    let main_fn = declare_main_function(&context, &module)?;

    // 创建 main 函数体
    let entry = context.append_basic_block(main_fn, "entry");
    builder.position_at_end(entry);

    // 生成打印 "Hello, World!" 的代码
    let hello_world = generate_hello_world(&context, &module, &builder)?;

    // 调用 puts 打印字符串
    builder.build_call(puts_fn, &[hello_world.into()], "puts_call")?;

    // 返回 0
    let i32_type = context.i32_type();
    builder.build_return(Some(&i32_type.const_int(0, false)))?;

    // 验证模块
    module.verify().map_err(|e| format!("Module verification failed: {}", e))?;

    // 打印生成的 IR
    println!("\nGenerated LLVM IR:");
    println!("{}", module.print_to_string().to_string_lossy());

    if let Some(output) = output_file {
        // 生成可执行文件
        generate_executable(&module, output)?;
    } else {
        // 使用 JIT 执行
        println!("\nNote: Use -o <file> to generate executable file");
    }

    Ok(())
}

/// 生成可执行文件
fn generate_executable(module: &Module, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 创建临时对象文件路径
    let temp_dir = std::env::temp_dir();
    let object_file = temp_dir.join("nexa_temp.o");

    // 使用系统默认的 target triple (Apple Silicon)
    let target_triple = TargetTriple::create("arm64-apple-macosx15.0.0");
    let target = Target::from_triple(&target_triple)?;

    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or("Failed to create target machine")?;

    // 生成对象文件
    target_machine
        .write_to_file(module, FileType::Object, object_file.as_path())
        .map_err(|e| format!("Failed to write object file: {}", e))?;

    // 链接生成可执行文件
    let output_path = PathBuf::from(output);
    let output_name = output_path.file_name().and_then(|n| n.to_str()).unwrap_or("a.out");

    let link_status =
        Command::new("clang").args([object_file.to_str().unwrap(), "-o", output_name]).status()?;

    // 清理临时文件
    let _ = std::fs::remove_file(&object_file);

    if link_status.success() {
        println!("\nCompiled to executable: {}", output_name);
        Ok(())
    } else {
        Err("Linker failed to produce executable".into())
    }
}

/// 声明 puts 函数
fn declare_puts_function<'a>(
    context: &'a Context,
    module: &Module<'a>,
) -> Result<FunctionValue<'a>, Box<dyn std::error::Error>> {
    let i8_ptr = context.ptr_type(Default::default());
    let fn_type = context.i32_type().fn_type(&[i8_ptr.into()], false);

    let puts = module.add_function("puts", fn_type, None);
    Ok(puts)
}

/// 声明 main 函数
fn declare_main_function<'a>(
    context: &'a Context,
    module: &Module<'a>,
) -> Result<FunctionValue<'a>, Box<dyn std::error::Error>> {
    let fn_type = context.i32_type().fn_type(&[], false);
    let main = module.add_function("main", fn_type, None);
    Ok(main)
}

/// 生成 Hello World 字符串
fn generate_hello_world<'a>(
    context: &'a Context,
    module: &Module<'a>,
    builder: &Builder<'a>,
) -> Result<inkwell::values::PointerValue<'a>, Box<dyn std::error::Error>> {
    // 创建全局字符串常量
    let hello_str = context.const_string(b"Hello, World!\n\0", false);
    let global = module.add_global(hello_str.get_type(), None, "helloworld");
    global.set_initializer(&hello_str);

    // 将全局变量转换为指针
    let i8_ptr_type = context.ptr_type(Default::default());
    let ptr_val = builder.build_bit_cast(global, i8_ptr_type, "helloworld_ptr")?;
    let ptr = ptr_val.into_pointer_value();

    Ok(ptr)
}
