//! LLVM 类型映射
//!
//! 提供 TS/Nexa 类型到 LLVM 类型的映射（后续对接类型检查器）。

use inkwell::context::Context;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::AddressSpace;

/// 将前端类型映射到 LLVM 类型的映射器
///
/// 当前提供基础类型的 LLVM 表示，后续对接 nexatypecheck 的 TsType。
pub struct TypeMapper<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> TypeMapper<'ctx> {
    /// 创建类型映射器
    #[must_use]
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// 获取 LLVM i32 类型
    #[must_use]
    pub fn i32_type(&self) -> inkwell::types::IntType<'ctx> {
        self.context.i32_type()
    }

    /// 获取 LLVM i64 类型
    #[must_use]
    pub fn i64_type(&self) -> inkwell::types::IntType<'ctx> {
        self.context.i64_type()
    }

    /// 获取 LLVM i1 类型（布尔）
    #[must_use]
    pub fn i1_type(&self) -> inkwell::types::IntType<'ctx> {
        self.context.custom_width_int_type(1)
    }

    /// 获取 LLVM void 类型
    #[must_use]
    pub fn void_type(&self) -> inkwell::types::VoidType<'ctx> {
        self.context.void_type()
    }

    /// 获取 i8* 类型（用于 string / 指针）
    #[must_use]
    pub fn i8_ptr_type(&self) -> inkwell::types::PointerType<'ctx> {
        self.context.ptr_type(AddressSpace::default())
    }

    /// 获取通用指针类型（默认地址空间）
    #[must_use]
    pub fn ptr_type(&self, addr_space: AddressSpace) -> inkwell::types::PointerType<'ctx> {
        self.context.ptr_type(addr_space)
    }

    /// 用于函数签名的 i8* 元数据类型
    #[must_use]
    pub fn i8_ptr_metadata(&self) -> BasicMetadataTypeEnum<'ctx> {
        self.i8_ptr_type().into()
    }
}
