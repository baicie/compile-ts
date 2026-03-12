//! 目标代码生成与链接
//!
//! 将 LLVM 模块写入对象文件并链接为可执行文件，支持主机目标检测。

use inkwell::module::Module;
use inkwell::targets::{CodeModel, FileType, RelocMode, Target, TargetMachine, TargetTriple};
use inkwell::OptimizationLevel;
use std::path::Path;
use std::process::Command;

/// 返回当前主机对应的 LLVM target triple 字符串
///
/// 用于在未指定 `--target` 时生成本机可执行文件。
#[must_use]
pub fn host_target_triple() -> String {
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

/// 使用主机目标将模块编译为对象文件并链接为可执行文件
///
/// # Errors
/// 目标初始化、对象文件写入或链接失败时返回错误
pub fn generate_executable(
    module: &Module,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let triple_str = host_target_triple();
    let target_triple = TargetTriple::create(&triple_str);
    let target = Target::from_triple(&target_triple)?;

    let target_machine: TargetMachine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or("Failed to create target machine")?;

    let temp_dir = std::env::temp_dir();
    let object_file = temp_dir.join("nexa_temp.o");

    target_machine
        .write_to_file(module, FileType::Object, object_file.as_path())
        .map_err(|e| format!("Failed to write object file: {}", e))?;

    let output_path_buf = Path::new(output_path);

    let link_status =
        Command::new("clang").arg(&object_file).arg("-o").arg(output_path_buf).status()?;

    let _ = std::fs::remove_file(&object_file);

    if link_status.success() {
        Ok(())
    } else {
        Err("Linker failed to produce executable".into())
    }
}
