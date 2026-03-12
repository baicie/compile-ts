#!/bin/bash
# 格式化脚本 - 格式化代码

set -e

echo "=== 代码格式化脚本 ==="

# 格式化代码
echo "格式化代码..."
cargo fmt

# 添加格式化后的文件到暂存区
echo "添加格式化后的文件..."
git add .

echo "=== 格式化完成 ==="
