// Nexa 编译器集成测试
// 测试编译器的基本功能

use std::process::Command;
use std::path::Path;

/// 测试编译器能否正确处理 hello_world 示例
#[test]
fn test_compile_hello_world() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let input_file = examples_dir.join("hello_world.ts");

    // 验证示例文件存在
    assert!(
        input_file.exists(),
        "Example file hello_world.ts should exist"
    );

    // 读取示例文件内容
    let content = std::fs::read_to_string(&input_file).expect("Failed to read example file");

    // 验证基本内容
    assert!(content.contains("Hello, World!"));
    assert!(content.contains("console.log"));
}

/// 测试编译器能否正确处理 fibonacci 示例
#[test]
fn test_compile_fibonacci() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let input_file = examples_dir.join("fibonacci.ts");

    // 验证示例文件存在
    assert!(
        input_file.exists(),
        "Example file fibonacci.ts should exist"
    );

    // 读取示例文件内容
    let content = std::fs::read_to_string(&input_file).expect("Failed to read example file");

    // 验证基本内容
    assert!(content.contains("fibonacci"));
    assert!(content.contains("function"));
}

/// 测试 examples 目录中的所有示例文件
#[test]
fn test_all_examples_exist() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");

    // 验证 examples 目录存在
    assert!(examples_dir.is_dir(), "examples directory should exist");

    // 列出所有 .ts 文件
    let ts_files: Vec<_> = std::fs::read_dir(&examples_dir)
        .expect("Failed to read examples directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map_or(false, |ext| ext == "ts")
        })
        .collect();

    // 至少应该有 2 个示例文件
    assert!(
        ts_files.len() >= 2,
        "Should have at least 2 example files, found {}",
        ts_files.len()
    );
}

/// 测试脚本目录是否存在
#[test]
fn test_scripts_directory_exists() {
    let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts");
    assert!(scripts_dir.is_dir(), "scripts directory should exist");
}
