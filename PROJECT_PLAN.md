# Nexa 语言编译器开发计划书

## 一、项目背景与目标

### 1.1 项目缘起

- **发起人**: 个人开发者 + AI 辅助
- **初衷**: 研究 + 开源玩
- **性质**: 长期项目
- **技术储备**: 少许前端编译器经验，Rust/JS/TS 熟悉

### 1.2 现有基础

项目源自 `compile-ts` (https://github.com/baicie/compile-ts)，已完成：

| 模块       | 状态    | 说明                            |
| ---------- | ------- | ------------------------------- |
| TS 解析器  | ✅ 完整 | 词法分析、AST 生成              |
| 类型检查   | ✅ 完整 | 接口、泛型、联合/交叉、条件类型 |
| 内存分配器 | ✅ 基础 | nexa_allocator                  |
| 数据结构   | ✅ 基础 | CodeBuffer 等                   |
| 代码生成   | ❌ 缺失 | AST → LLVM IR                   |
| 编译输出   | ❌ 缺失 | 可执行文件生成                  |

### 1.3 核心目标

```
长期目标: 打造一门 "TS 语法的 Zig" 语言
    ├── 继承 TS 语法风格（类型标注、interface、enum）
    ├── 显式内存管理（类似 Zig，无隐藏 alloc/free）
    ├── defer 关键词（作用域结束时自动释放资源）
    └── 编译为多平台（利用 LLVM）
```

---

## 二、技术架构

### 2.1 整体架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Nexa 编译器架构                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   源代码 (.ts)                                                         │
│        │                                                                    │
│        ▼                                                                    │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────────────────┐   │
│   │   Lexer      │───▶│   Parser     │───▶│   AST                    │   │
│   │  (词法分析)   │    │  (语法分析)   │    │  (抽象语法树)            │   │
│   └──────────────┘    └──────────────┘    └──────────────────────────┘   │
│                                                      │                     │
│                                                      ▼                     │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                        类型检查 (TypeChecker)                       │  │
│   │   - 类型推导    - 接口检查    - 泛型解析    - 联合/交叉类型         │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                      │                     │
│                                                      ▼                     │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                         代码生成 (CodeGen)                          │  │
│   │   ┌─────────────┐  ┌──────────────┐  ┌────────────────────────┐   │  │
│   │   │ IRBuilder   │─▶│ LLVMBuilder │─▶│ Optimizer    │─▶│ TargetCodeGen         │   │  │
│   │   │ (中间表示)   │  │ (LLVM IR)    │  │ (优化 passes)│  │ (机器码/字节码)       │   │  │
│   │   └─────────────┘  └──────────────┘  └─────────────┘  └────────────────────────┘   │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                      │                     │
│        ▼                              ▼▼▼▼▼▼                               │
│   可执行文件 ◀─────────────────────────────┘                                │
│   - ELF (Linux)    - Mach-O (macOS)    - PE (Windows)    - WASM          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 核心技术选型

| 层级         | 技术选型       | 理由                      |
| ------------ | -------------- | ------------------------- |
| **前端**     | 自研 (Rust)    | 已有 TS 解析器基础        |
| **IR**       | 自己的 IR 层   | 解耦前端和后端，便于优化  |
| **优化器**   | LLVM Passes    | 利用 LLVM 内置优化        |
| **后端**     | LLVM           | 成熟、多平台、活跃维护    |
| **语言绑定** | inkwell (Rust) | 官方推荐的 Rust LLVM 绑定 |

### 2.3 项目目录结构

```
nexa/
├── Cargo.toml                    # Workspace 配置
├── compiler/
│   ├── nexa/                       # 主编译器入口
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   ├── nexa_allocator/             # 内存分配器
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── allocator.rs
│   │       ├── address.rs
│   │       └── boxed.rs
│   ├── nexa_data_structures/       # 数据结构
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── code_buffer.rs
│   ├── nexa_parser/                # 解析器 (新建)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── lexer.rs
│   │       ├── parser.rs
│   │       └── ast.rs
│   ├── nexa_typecheck/             # 类型检查器 (新建)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── checker.rs
│   │       └── types.rs
│   ├── nexa_codegen/               # 代码生成器 (新建)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── context.rs
│   │       ├── module.rs
│   │       ├── types.rs
│   │       ├── value.rs
│   │       ├── function.rs
│   │       ├── builder.rs
│   │       ├── optimizer.rs        # LLVM 优化 passes
│   │       └── target.rs
│   └── nexa_std/                   # 标准库
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── io.rs
│           └── memory.rs
├── std/                          # 运行时 std 源码
│   ├── io.ts
│   └── memory.ts
└── tests/
    └── integration/
        └── basic.rs
```

---

## 三、阶段规划

### 3.1 阶段一：LLVM 集成基础

**时间**: 第 1-3 个月

**目标**: 搭建 LLVM 基础设施，实现 Hello World 编译

#### 1.1.1 环境搭建

| 任务                   | 说明           | 预估工作量 |
| ---------------------- | -------------- | ---------- |
| 添加 inkwell 依赖      | Rust LLVM 绑定 | 1 周       |
| 配置 LLVM 工具链       | 建议预编译版本 | 1 周       |
| 创建 nexacodegen crate | 代码生成模块   | 1 周       |

#### 1.1.2 核心模块设计

```rust
// nexacodegen/src/types.rs - 类型映射
pub struct TypeMapper<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> TypeMapper<'ctx> {
    pub fn map_ts_type(&self, ts_type: &TsType) -> &<'ctx, Dynamic> {
        match ts_type {
            TsType::Number => self.context.i32_type(),
            TsType::String => self.context.i8_type().ptr_type(AddressSpace::default()),
            TsType::Boolean => self.context.i1_type(),
            TsType::Void => self.context.void_type(),
            TsType::Array(t) => self.map_array_type(t),
            TsType::Object(fields) => self.map_struct_type(fields),
            // ...
        }
    }
}
```

#### 1.1.3 类型映射表

| TS/Nexa 类型     | LLVM 类型    | 备注     |
| ---------------- | ------------ | -------- |
| `i32` / `number` | `i32`        |          |
| `i64`            | `i64`        |          |
| `f32`            | `float`      |          |
| `f64`            | `double`     |          |
| `boolean`        | `i1`         |          |
| `void`           | `void`       |          |
| `string`         | `i8*`        | 指针     |
| `null`           | `i8*` (null) |          |
| `undefined`      | `i8*` (null) |          |
| `any`            | `i8*`        | 指针     |
| `T[]`            | `T*`         | 数组指针 |
| `struct`         | `{ ... }`    | 结构体   |

#### 1.1.4 LLVM 优化配置

| 优化级别 | 适用场景 | 说明                   |
| -------- | -------- | ---------------------- |
| -O0      | 调试模式 | 无优化，便于调试       |
| -O1      | 快速编译 | 基础优化，编译速度优先 |
| -O2      | 发布模式 | 标准优化               |
| -O3      | 极致性能 | 激进优化               |
| -Oz      | 体积优先 | 最小化二进制体积       |

常用优化 Pass：

- **IPO**: 跨过程优化
- **Inlining**: 函数内联
- **CFSimplify**: 控制流简化
- **GVN**: 全局值编号（常量传播）
- **DCE**: 死代码消除
- **LICM**: 循环不变代码外提
- **LoopUnroll**: 循环展开

#### 1.1.5 里程碑 M1: Hello World

```ts
// hello.ts
fn main() {
    println("Hello, World!");
}
```

预期输出：

- 生成正确 LLVM IR
- 编译为可执行文件
- 运行输出 "Hello, World!"

---

### 3.2 阶段二：代码生成实现

**时间**: 第 4-6 个月

**目标**: 实现表达式、控制流、函数的代码生成

#### 2.1 表达式生成

| 优先级 | 特性     | 说明               |
| ------ | -------- | ------------------ |
| P0     | 字面量   | 数字、字符串、布尔 |
| P0     | 变量引用 | 局部变量、全局变量 |
| P0     | 算术运算 | + - \* / %         |
| P1     | 比较运算 | == != < > <= >=    |
| P1     | 逻辑运算 | && \|\| !          |
| P1     | 函数调用 | 普通函数调用       |
| P2     | 位运算   | & \| ^ ~ << >>     |

#### 2.2 控制流生成

| 优先级 | 特性           | 说明     |
| ------ | -------------- | -------- |
| P0     | if/else        | 条件分支 |
| P0     | while          | 循环     |
| P0     | return         | 返回值   |
| P1     | for            | 计数循环 |
| P1     | break/continue | 循环控制 |
| P2     | match          | 模式匹配 |

#### 2.3 函数生成

| 优先级 | 特性     | 说明                          |
| ------ | -------- | ----------------------------- |
| P0     | 函数定义 | fn add(a: i32, b: i32) -> i32 |
| P0     | 参数传递 | 按值传递                      |
| P0     | 返回值   | 处理 void 和非 void           |
| P1     | 局部变量 | 栈分配                        |
| P2     | 闭包     | 高优先级延后                  |

#### 2.4 结构体生成

| 优先级 | 特性        | 说明          |
| ------ | ----------- | ------------- |
| P0     | struct 定义 | 内存布局      |
| P0     | 字段访问    | a.x           |
| P1     | 方法调用    | obj.method()  |
| P2     | 继承        | class extends |

#### 2.5 里程碑 M2-M3

```
M2: 基本表达式和函数 (第 4 个月)
    - 支持加减乘除、比较、逻辑运算
    - 支持函数定义和调用

M3: 控制流和循环 (第 6 个月)
    - 支持 if/else、while、for
    - 支持 break/continue、return
```

---

### 3.3 阶段三：编译输出

**时间**: 第 7-9 个月

**目标**: 生成可执行文件，支持多平台

#### 3.1 目标文件生成

| 平台    | 格式   | 状态     |
| ------- | ------ | -------- |
| Linux   | ELF    | 目标     |
| macOS   | Mach-O | 目标     |
| Windows | PE     | 目标     |
| Web     | WASM   | 长期目标 |

#### 3.2 链接支持

| 类型     | 说明                |
| -------- | ------------------- |
| 静态链接 | 默认，链接 libc     |
| 动态链接 | 运行时加载 .so/.dll |
| 无链接   | 直接输出汇编        |

#### 3.3 标准库基础

```nexa
// 基础 IO
fn println(s: string): void
fn print(s: string): void
fn readln(): string

// 内存管理
fn malloc(size: usize): *u8
fn free(ptr: *u8): void

// 基础系统
fn exit(code: i32): void
fn sleep(ms: u32): void
```

#### 3.4 里程碑 M4: 生成可执行文件 (第 9 个月)

- 支持 `nexac hello.ts -o hello` 生成可执行文件
- 支持多平台交叉编译

#### 3.5 错误处理与诊断

编译器错误信息和运行时错误处理：

```
编译器错误输出示例:
error[001]: undefined variable 'x'
  ┌── test.nexa:5:10
  │
5 │     println(x);
  │          ^ not found in this scope
```

| 特性         | 说明                       |
| ------------ | -------------------------- |
| 源码位置     | 行号、列号、文件路径       |
| 错误上下文   | 显示问题代码的周围上下文   |
| 错误类型     | 词法、语法、类型、语义错误 |
| 运行时 panic | 栈展开、错误信息输出       |
| 调试符号     | DWARF 调试信息 (可选)      |

#### 3.6 调试信息 (DWARF)

生成可执行文件时支持调试信息：

| 特性     | 说明                   |
| -------- | ---------------------- |
| 行号信息 | 源码到机器码的映射     |
| 变量信息 | 本地变量作用域和位置   |
| 类型信息 | 结构体、枚举等类型定义 |
| 断点支持 | gdb/lldb 断点设置      |
| 栈展开   | 运行时栈追踪           |

#### 3.7 工具链设计

CLI 接口设计：

```bash
# 编译
nexac compile <input> -o <output>

# 选项
nexac compile input.nexa -o output \
    --opt-level 2          # 优化级别 (-O0/-O1/-O2/-O3/-Oz)
    --target x86_64-linux  # 目标平台
    --output-type exe      # 输出类型 (exe/asm/llvm-ir)
    --debug                # 包含调试信息
    --static               # 静态链接

# 常用命令
nexac run <file>           # 编译并运行
nexac check <file>         # 类型检查，不生成代码
nexac fmt <file>           # 代码格式化
nexac doc <file>           # 生成文档
```

| 命令    | 说明         |
| ------- | ------------ |
| compile | 编译源文件   |
| run     | 编译并运行   |
| check   | 类型检查     |
| fmt     | 代码格式化   |
| doc     | 生成文档     |
| version | 显示版本信息 |

---

### 3.4 阶段四：自举准备

**时间**: 第 10-12 个月

**目标**: 设计语言子集，实现自举

#### 4.1 Nexa0 子集设计

最小可自举子集，约 2000 行代码：

```
Nexa0 语言规范:
├── 基础类型
│   ├── i32, i64, f32, f64
│   └── bool
├── 控制流
│   ├── if/else
│   └── while
├── 函数
│   ├── 函数定义和调用
│   └── 递归
├── 结构体
│   ├── struct 定义
│   └── 字段访问
└── 内存
    ├── 栈分配
    └── 指针 (仅基本操作)
```

#### 4.2 自举路径

```
Year 1 结束:
    Nexa 编译器 (Rust 实现) → 编译完整 Nexa 语言
              │
              ▼
    Nexa 编译器 (Nexa0 子集，用 Rust 写)
              │
              ▼
    Nexa 编译器 (Nexa0 子集，用 Nexa 写) ← 自举完成！
              │
              ▼
    完整 Nexa 编译器 (用 Nexa 写)
```

#### 4.3 里程碑 M5-M6

```
M5: Nexa0 子集完成 (第 10 个月)
    - 语言子集语法完整
    - 可以自编译

M6: 初步自举 (第 12 个月)
    - Nexa 编译器可用 Nexa 编写
    - 具备后续扩展基础
```

---

## 四、差异化特性设计

### 4.1 运行时设计

虽然强调"无 GC"和"显式内存管理"，但高级类型仍需要运行时支持：

#### 4.1.1 字符串表示

```nexa
// 字符串内部表示 (UTF-8)
struct String {
    ptr: *u8,    // 指向堆内存
    len: usize,  // 字节长度
    cap: usize,  // 容量
}
```

#### 4.1.2 数组表示

```nexa
// 动态数组表示
struct Array<T> {
    ptr: *T,     // 指向堆内存
    len: usize,  // 元素数量
    cap: usize,  // 容量
}
```

#### 4.1.3 闭包表示

```nexa
// 闭包捕获上下文
struct Closure<T> {
    fn_ptr: *const (),     // 函数指针
    ctx: *const CapturedContext,  // 捕获的上下文
}
```

#### 4.1.4 运行时初始化

```nexa
// 运行时初始化入口
fn __runtime_init() {
    // 初始化堆分配器
    allocator_init();

    // 初始化标准库
    std_init();

    // 调用用户 main
    main();
}
```

### 4.2 内存管理模型

**借鉴 Zig 显式内存管理 + defer 关键词**

```nexa
// 显式内存分配 - 类似 Zig
fn create_buffer(size: usize): *u8 {
    const buf = malloc(size);  // 显式分配
    // 使用 buf...
    return buf;
}

// defer 关键词 - 作用域结束时自动释放
fn read_file(path: string): []u8 {
    const fd = open(path);
    defer close(fd);  // 函数结束自动调用 close(fd)

    const data = read(fd);
    return data;  // close(fd) 会在 return 之前自动执行
}

// 资源清理示例
fn process_request(req: Request): Response {
    const conn = connect(req.server);
    defer disconnect(conn);  // 确保连接被关闭

    if (req.timeout > 0) {
        defer set_timeout(conn, req.timeout);
    }

    return send(conn, req.data);
}

// 错误处理结合 defer
fn write_log(path: string, data: []u8) !void {
    const file = try open_file(path);
    defer close(file);  // 即使出错也会关闭文件

    try write(file, data);
}
```

**核心原则:**

- 所有内存分配都是显式的 (`malloc`, `alloc`, `new`)
- 所有资源都需要手动释放 (`free`, `close`)
- `defer` 确保资源在作用域结束时一定被释放
- 无隐藏的 GC，无隐式的 drop/cleanup

### 4.2 并发模型

**借鉴 Go Goroutine + Actor**

```nexa
// 异步函数
async fn fetch(url: string): Response {
    let resp = await http_get(url);
    resp
}

// 并发 spawn
spawn {
    do_something();
}
```

### 4.3 互操作性

**完美 C FFI**

```nexa
// 导入 C 函数
extern "C" {
    fn printf(format: *const i8, ...) -> i32;
}

// 导出给 C
export fn my_function() {
    // ...
}
```

---

## 五、关键里程碑汇总

| 里程碑 | 时间       | 目标             | 产出                   |
| ------ | ---------- | ---------------- | ---------------------- |
| M1     | 第 1 个月  | Hello World 编译 | 可运行 hello world     |
| M2     | 第 4 个月  | 基本表达式和函数 | 支持加减乘除、函数调用 |
| M3     | 第 6 个月  | 控制流和循环     | if/while/for/return    |
| M4     | 第 9 个月  | 生成可执行文件   | 支持多平台编译         |
| M5     | 第 10 个月 | Nexa0 子集完成   | 最小语言子集           |
| M6     | 第 12 个月 | 初步自举         | 用 Nexa 写 Nexa        |

### 里程碑检查清单

```
M1: Hello World
    □ inkwell 集成完成
    □ 生成正确 LLVM IR
    □ 编译为可执行文件
    □ 运行输出 "Hello, World!"

M2: 基本表达式和函数
    □ 支持加减乘除运算
    □ 支持比较运算
    □ 支持逻辑运算
    □ 支持函数定义和调用
    □ 支持局部变量

M3: 控制流和循环
    □ 支持 if/else 条件分支
    □ 支持 while 循环
    □ 支持 for 计数循环
    □ 支持 break/continue
    □ 支持 return 返回值

M4: 生成可执行文件
    □ 支持 ELF (Linux) 输出
    □ 支持 Mach-O (macOS) 输出
    □ 支持 PE (Windows) 输出
    □ 支持链接静态库
    □ CLI 工具完整

M5: Nexa0 子集完成
    □ Nexa0 语法完整
    □ 自举编译器可编译 Nexa0
    □ 生成正确可执行文件

M6: 初步自举
    □ Nexa 编译器用 Nexa 编写
    □ 可编译自身
    □ 具备后续扩展基础
```

### 预估代码量

```
阶段              | 预估代码量 | 说明
---------------- | ---------- | ---------------------------
Nexa0 子集       | ~2000 行   | 最小自举语言
完整编译器 (Rust)| ~8000 行   | 包含所有优化
标准库 (Nexa)    | ~3000 行   | 基础库函数
总计 (第一年)     | ~13000 行  |
```

---

## 六、技术风险与应对

### 6.1 主要风险

| 风险            | 影响 | 应对策略               |
| --------------- | ---- | ---------------------- |
| LLVM 集成复杂度 | 高   | 分阶段，先跑通再优化   |
| 类型系统扩展    | 中   | 复用现有 TS 类型检查   |
| 自举困难        | 高   | 设计最小子集 Nexa0     |
| 性能优化        | 中   | 后期优化，先保证正确性 |
| 调试信息生成    | 中   | 借助 LLVM DWARF 支持   |
| 运行时设计      | 中   | 简化设计，渐进式扩展   |
| 错误信息质量    | 低   | 持续迭代改进           |

### 6.2 备选方案

- **如果 LLVM 过重**: 考虑先实现解释器，再逐步迁移到 AOT
- **如果自举困难**: 保持 Rust 实现，长期用 Rust 开发
- **如果多平台复杂**: 先专注 Linux，其他平台后续

---

## 七、开发实践建议

### 7.1 测试策略

```
单元测试:
    └── 每个 IR 生成函数 → 对应测试用例

集成测试:
    └── 完整编译流程 → 端到端测试

快照测试:
    └── LLVM IR 输出 → 版本对比

Fuzzing 测试:
    └── 随机生成 AST/代码 → 验证编译器不崩溃

性能基准测试:
    └── 编译速度、生成代码性能 → 持续监控

属性测试 (Property-based):
    └── 生成随机输入 → 验证输出符合预期规则
```

### 7.2 语言规范文档

语言规范是编译器开发的权威参考：

| 文档       | 说明                       |
| ---------- | -------------------------- |
| 词法规范   | 关键字、标识符、字面量规则 |
| 语法规范   | EBNF 语法定义              |
| 类型系统   | 类型推导规则、类型兼容性   |
| 语义规范   | 作用域、生命周期、内存模型 |
| 标准库文档 | 内置函数和行为说明         |

### 7.3 包管理器设计 (长期)

长期来看需要包管理支持：

```
nexa.toml:
    [package]
    name = "my-package"
    version = "0.1.0"

    [dependencies]
    json = "nexa:json@^1.0.0"
    http = "nexa:http@^2.0.0"

    [dev-dependencies]
    test = "nexa:test@^1.0.0"
```

| 功能     | 说明                     |
| -------- | ------------------------ |
| 依赖解析 | 语义版本、依赖冲突检测   |
| 镜像源   | 官方 registry + 镜像支持 |
| 缓存管理 | 本地缓存、离线编译       |
| 发布流程 | 打包、上传、版本管理     |

### 7.4 代码审查

- 每周 1 次 PR review
- 核心设计需文档化
- 保持提交历史清晰

### 7.5 版本规划

```
v0.1.0: Hello World
v0.2.0: 基本表达式
v0.3.0: 控制流
v0.4.0: 可执行文件
v0.5.0: Nexa0 自举
v1.0.0: 完整语言
```

### 7.6 进阶特性 (长期)

#### 7.6.1 LSP 支持

Language Server Protocol 实现 IDE 集成：

```
功能:
    - 代码补全
    - 跳转到定义
    - 查找引用
    - 诊断信息
    - 代码格式化
    - 重构支持
```

#### 7.6.2 REPL 交互式解释器

交互式编程环境：

```
$ nexac repl
Nexa> let x = 1 + 2
3
Nexa> fn add(a: i32, b: i32) -> i32 { a + b }
fn add(a: i32, b: i32) -> i32
Nexa> add(1, 2)
3
```

#### 7.6.3 内联汇编

类似 Zig 的内联汇编支持：

```nexa
fn syscall() -> i64 {
    asm volatile(
        "movq $$60, %rax",
        "movq $$42, %rdi",
        "syscall",
        :: "rdi", "rax"
    ) -> i64
}
```

#### 7.6.4 编译时元编程

支持编译时代码生成：

```nexa
// 宏展开
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {
        if $left != $right {
            panic!("assertion failed: {} != {}", $left, $right)
        }
    };
}

// 使用
assert_eq!(1 + 1, 2);
```

#### 7.6.5 代码分析工具

```
nexa-analyze:
    - 复杂度分析
    - 死代码检测
    - 内存泄漏检测
    - 性能热点分析
    - 安全漏洞扫描
```

---

## 八、结语

这是一个长期项目，核心价值在于：

1. **学习**: 深入理解编译器原理
2. **创造**: 实现自己的语言设计理念
3. **贡献**: 为开源社区贡献 Nexa 语言
4. **成长**: 与 AI 协作开发的最佳实践

> "Code is poetry, Language is art."

期待 Nexa 语言成为你技术生涯中的重要作品。

---

_文档版本: v1.1_
_创建时间: 2026-03-11_
_更新周期: 每月更新_
