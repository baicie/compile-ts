compiler/tsc-codegen-llvm/
├── Cargo.toml
├── src/
    ├── lib.rs           // 主入口
    ├── generator.rs     // 代码生成器核心
    ├── expression.rs    // 表达式生成
    ├── statement.rs     // 语句生成
    ├── function.rs      // 函数生成
    ├── types.rs         // 类型系统
    ├── gc.rs            // 垃圾回收
    └── utils.rs         // 工具函数