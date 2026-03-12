#!/bin/bash
# 测试脚本 - 运行所有测试

set -e

echo "=== Nexa 编译器测试脚本 ==="

# 运行单元测试
echo "运行单元测试..."
cargo test

# 运行集成测试
echo "运行集成测试..."
cargo test --test compiler_test

# 运行文档测试
echo "运行文档测试..."
cargo test --doc

echo "=== 测试完成 ==="
