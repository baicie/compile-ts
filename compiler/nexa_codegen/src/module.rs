//! 模块级代码生成
//!
//! 声明运行时函数（如 puts）、main，以及全局常量（如字符串）。

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

/// 声明 C 标准库 `puts(i8*) -> i32`
#[must_use]
pub fn declare_puts<'a>(context: &'a Context, module: &'a Module<'a>) -> FunctionValue<'a> {
    let i8_ptr = context.ptr_type(Default::default());
    let fn_type = context.i32_type().fn_type(&[i8_ptr.into()], false);
    module.add_function("puts", fn_type, None)
}

/// 声明 `main() -> i32`
#[must_use]
pub fn declare_main<'a>(context: &'a Context, module: &'a Module<'a>) -> FunctionValue<'a> {
    let fn_type = context.i32_type().fn_type(&[], false);
    module.add_function("main", fn_type, None)
}

/// 在模块中创建全局字符串常量，返回可作为 i8* 使用的指针值
///
/// # Errors
/// 构建 bitcast 失败时返回错误
pub fn add_global_string<'a>(
    context: &'a Context,
    module: &'a Module<'a>,
    builder: &'a Builder<'a>,
    content: &[u8],
    name: &str,
) -> Result<inkwell::values::PointerValue<'a>, inkwell::builder::BuilderError> {
    let zero = content.last().copied() != Some(0);
    let llvm_str = context.const_string(content, zero);
    let global = module.add_global(llvm_str.get_type(), None, name);
    global.set_initializer(&llvm_str);

    let i8_ptr_type = context.ptr_type(Default::default());
    let ptr_name = format!("{name}_ptr");
    let ptr_val = builder.build_bit_cast(global, i8_ptr_type, &ptr_name)?;
    Ok(ptr_val.into_pointer_value())
}
