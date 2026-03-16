---
description: Nexa 编译器 Rust 开发代理配置
globs: **/*.rs
alwaysApply: false
---

# Nexa 编译器开发代理

## 项目概述

你是 Nexa 编译器的 AI 开发助手。Nexa 是一个用 Rust 编写的编译器项目，用于将 TypeScript 语法编译为可执行文件。

## 项目结构

```
compiler/
├── nexac/              # 主编译器 crate
│   ├── src/
│   │   ├── main.rs     # 入口点
│   │   ├── lib.rs      # 库入口
│   │   ├── lexer/      # 词法分析器
│   │   ├── parser/     # 语法分析器
│   │   ├── ast/        # AST 定义
│   │   ├── codegen/    # 代码生成
│   │   └── ...
│   └── Cargo.toml
├── nexa_allocator/     # 自定义内存分配器
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
└── ...
```

## 核心模块职责

### 1. 词法分析器 (lexer)

- **位置**: `compiler/nexac/src/lexer/`
- **职责**: 将源代码字符串转换为 token 序列
- **关键类型**: `Lexer`, `Token`, `TokenKind`
- **返回值**: `Result<Vec<Token>, LexerError>`

### 2. 语法分析器 (parser)

- **位置**: `compiler/nexac/src/parser/`
- **职责**: 将 token 序列转换为抽象语法树 (AST)
- **方法**: 递归下降解析
- **要求**: 每个 AST 节点必须实现 `Span` trait

### 3. 代码生成器 (codegen)

- **位置**: `compiler/nexac/src/codegen/`
- **职责**: 将 AST 转换为 LLVM IR
- **依赖**: 使用 inkwell 绑定 LLVM
- **输出**: LLVM IR 或目标机器码

### 4. 内存分配器 (allocator)

- **位置**: `compiler/nexa_allocator/`
- **职责**: 自定义内存管理
- **风格**: 遵循 Zig 风格的显式内存管理
- **接口**: `alloc` / `dealloc`

## Rust 开发规范

### 代码风格

1. **格式化**: 使用 `rustfmt`，配置见 `.rustfmt.toml`
2. **检查**: 运行 `cargo clippy -- -D warnings`
3. **测试**: 运行 `cargo test`

### 命名规范

```rust
// 模块/函数/变量: snake_case
fn process_token() {}
let mut buffer = Vec::new();

// 结构体/枚举: PascalCase
pub struct Lexer<'a> {
    input: &'a str,
}

pub enum TokenKind {
    Identifier,
    Number,
}

// 常量: SCREAMING_SCREAMING_SCREAMING
const MAX_TOKEN_LENGTH: usize = 1024;
```

### 导入组织顺序

```rust
// 标准库 → 外部 crate → 项目内部模块
use std::collections::HashMap;

use bumpalo::Bump;

mod lexer;
mod parser;
```

## 最佳实践

### 所有权与借用

```rust
// 优先使用引用而非克隆
fn process_input(input: &str) -> Result<&str, Error> {
    // ...
}
```

### 错误处理

```rust
// 使用 thiserror 或自定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Lexer error: {0}")]
    LexerError(String),
    
    #[error("Parser error at line {line}: {message}")]
    ParserError { line: usize, message: String },
}

// 使用 ? 运算符传播错误
fn parse_file(path: &Path) -> Result<Ast, Error> {
    let content = std::fs::read_to_string(path)?;
    let tokens = lexer::tokenize(&content)?;
    let ast = parser::parse(tokens)?;
    Ok(ast)
}
```

### 性能优化

```rust
// 使用 arena 分配器减少分配
let arena = Bump::new();
let tokens = lexer::tokenize_with_arena(&content, &arena);

// 预先分配容量
let mut buffer = Vec::with_capacity(1024);

// 使用 &str 而非 String 当不需要所有权
fn parse_keyword(input: &str) -> Option<Keyword>;
```

## 文档要求

### 公共 API 必须文档化

```rust
/// 将源代码字符串 token 化
/// 
/// # Arguments
/// * `input` - 源代码字符串
/// 
/// # Errors
/// 返回 `LexerError` 当遇到无效字符
/// 
/// # Example
/// ```
/// let tokens = tokenize("let x = 1;").unwrap();
/// assert_eq!(tokens.len(), 5);
/// ```
pub fn tokenize(input: &str) -> Result<Vec<Token>, Error>;
```

### 模块级文档

```rust
//! 词法分析器模块
//! 
//! 负责将源代码转换为 Token 序列，支持以下词法元素：
//! - 标识符和关键字
//! - 数字字面量
//! - 字符串字面量
//! - 运算符和标点符号
```

## 开发工作流

1. **理解需求**: 阅读 `PROJECT_PLAN.md` 了解项目计划
2. **编写代码**: 遵循上述规范
3. **格式化**: `cargo fmt`
4. **检查**: `cargo clippy -- -D warnings`
5. **测试**: `cargo test`
6. **提交**: 确保所有检查通过，使用 Conventional Commits

## 提交规范

使用 Conventional Commits 格式：

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

类型：
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建过程或辅助工具变动

示例：
```bash
git commit -m "feat(lexer): 添加字符串字面量解析"
git commit -m "fix(parser): 修复数组类型解析错误"
```

## 常用命令

```bash
# 编译项目
cargo build

# 运行测试
cargo test

# 格式化代码
cargo fmt

# 运行 Clippy
cargo clippy -- -D warnings

# 查看文档
cargo doc --open

# 运行特定测试
cargo test lexer

# 发布构建
cargo build --release
```

## 关键依赖

- **inkwell**: LLVM 绑定
- **bumpalo**: Arena 内存分配
- **thiserror**: 错误类型定义
- **tracing**: 日志记录
- **cargo**: Rust 包管理器

## 交互指南

当用户请求帮助时：

1. **理解上下文**: 先阅读相关代码文件
2. **提供方案**: 给出具体的实现建议
3. **代码示例**: 提供可运行的代码示例
4. **解释原理**: 解释为什么这样做
5. **验证**: 建议如何验证解决方案

当你需要修改代码时：

1. 先读取相关文件了解现状
2. 遵循项目的代码风格
3. 添加适当的文档注释
4. 运行格式化、检查和测试
5. 确保提交信息符合规范
