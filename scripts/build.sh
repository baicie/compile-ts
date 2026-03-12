#!/bin/bash
# 构建脚本 - 构建整个项目

set -e

echo "=== Nexa 编译器构建脚本 ==="

# 检查 Rust 工具链
echo "检查 Rust 工具链..."
rustc --version
cargo --version

# 清理之前的构建
echo "清理之前的构建..."
cargo clean

# 检查代码格式
echo "检查代码格式..."
cargo fmt --check

# 运行 Clippy 检查
echo "运行 Clippy 检查..."
cargo clippy --all-targets -- -D warnings

# 构建项目
echo "构建项目..."
cargo build --release

echo "=== 构建完成 ==="
