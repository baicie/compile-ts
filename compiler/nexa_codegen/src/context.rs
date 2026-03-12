//! 代码生成上下文
//!
//! 持有 LLVM Context、Module、Builder，供整次编译使用。

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;

/// 单次编译的代码生成上下文
pub struct CodegenContext<'ctx> {
    /// LLVM 上下文
    pub context: &'ctx Context,
    /// 当前模块
    pub module: Module<'ctx>,
    /// IR 构建器
    pub builder: Builder<'ctx>,
}

impl<'ctx> CodegenContext<'ctx> {
    /// 创建新的代码生成上下文
    ///
    /// # Arguments
    /// * `context` - LLVM 上下文（由调用方创建并管理生命周期）
    /// * `module_name` - 模块名称，用于调试与符号
    #[must_use]
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        Self { context, module, builder }
    }

    /// 消费上下文并返回模块，便于在 IR 构建完成后进行验证与输出
    #[must_use]
    pub fn into_module(self) -> Module<'ctx> {
        self.module
    }
}
