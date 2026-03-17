//! Nexa 代码生成模块
//!
//! 将 AST 转换为 LLVM IR，支持生成对象文件与可执行文件。

mod module;
mod target;
mod types;

pub use module::{add_global_string, declare_main, declare_puts};
pub use target::{generate_executable, host_target_triple};
pub use types::TypeMapper;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::AddressSpace;
use nexa_parser::ast::*;
use std::collections::HashMap;

/// 代码生成器
/// 将 AST 转换为 LLVM IR
pub struct CodeGenerator<'ctx> {
    /// LLVM 上下文
    context: &'ctx Context,
    /// 当前模块
    module: Module<'ctx>,
    /// IR 构建器
    builder: Builder<'ctx>,
    /// 当前函数
    current_function: Option<FunctionValue<'ctx>>,
    /// 局部变量映射 (变量名 -> 指针值)
    variables: HashMap<String, PointerValue<'ctx>>,
    /// 变量类型映射 (变量名 -> LLVM 类型)
    variable_types: HashMap<String, inkwell::types::BasicTypeEnum<'ctx>>,
    /// 类型映射器
    #[allow(dead_code)]
    type_mapper: TypeMapper<'ctx>,
    /// Struct 类型定义映射
    struct_types: HashMap<String, inkwell::types::StructType<'ctx>>,
}

impl<'ctx> CodeGenerator<'ctx> {
    /// 创建新的代码生成器
    pub fn new(context: &'ctx Context, name: &str) -> Self {
        let module = context.create_module(name);
        let builder = context.create_builder();
        let type_mapper = TypeMapper::new(context);
        Self {
            context,
            module,
            builder,
            current_function: None,
            variables: HashMap::new(),
            variable_types: HashMap::new(),
            type_mapper,
            struct_types: HashMap::new(),
        }
    }

    /// 获取上下文
    pub fn context(&self) -> &'ctx Context {
        self.context
    }

    /// 将任意 BasicValue 转换为 i32
    fn coerce_to_i32(
        &mut self,
        value: inkwell::values::BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, CodegenError> {
        let i32_type = self.context.i32_type();
        match value {
            inkwell::values::BasicValueEnum::IntValue(v) => Ok(v),
            inkwell::values::BasicValueEnum::PointerValue(p) => {
                // 指针转换为 int
                Ok(self.builder().build_ptr_to_int(p, i32_type, "ptr_to_i32")?)
            },
            _ => Err(CodegenError { message: "Cannot coerce type to i32".to_string() }),
        }
    }

    /// 获取模块
    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    /// 获取构建器
    pub fn builder(&mut self) -> &mut Builder<'ctx> {
        &mut self.builder
    }

    /// 添加局部变量
    pub fn add_variable(&mut self, name: String, value: PointerValue<'ctx>) {
        self.variables.insert(name, value);
    }

    /// 获取局部变量
    pub fn get_variable(&self, name: &str) -> Option<PointerValue<'ctx>> {
        self.variables.get(name).copied()
    }

    /// 清除局部变量
    pub fn clear_variables(&mut self) {
        self.variables.clear();
        self.variable_types.clear();
    }

    /// 设置变量类型
    pub fn set_variable_type(&mut self, name: &str, ty: inkwell::types::BasicTypeEnum<'ctx>) {
        self.variable_types.insert(name.to_string(), ty);
    }

    /// 获取变量类型
    pub fn get_variable_type(&self, name: &str) -> Option<inkwell::types::BasicTypeEnum<'ctx>> {
        self.variable_types.get(name).copied()
    }

    /// 设置当前函数
    pub fn set_function(&mut self, func: FunctionValue<'ctx>) {
        self.current_function = Some(func);
    }

    /// 消费生成器并返回模块
    pub fn into_module(self) -> Module<'ctx> {
        self.module
    }

    /// 声明内置函数 (puts, console.log 等)
    pub fn declare_builtin_functions(&mut self) {
        let i8_ptr = self.context.ptr_type(AddressSpace::default());
        let i32_type = self.context.i32_type();

        // puts - C 标准库输出字符串
        let puts_type = i32_type.fn_type(&[i8_ptr.into()], false);
        self.module.add_function("puts", puts_type, None);

        // console.log (通过 puts 实现)
        let console_log_type = i32_type.fn_type(&[i8_ptr.into()], false);
        self.module.add_function("console.log", console_log_type, None);

        // scanf - C 标准库读取输入
        let scanf_type = i32_type.fn_type(&[i8_ptr.into()], true);
        self.module.add_function("scanf", scanf_type, None);

        // readln_i32 - 读取整数
        let readln_i32_type = i32_type.fn_type(&[], false);
        self.module.add_function("readln_i32", readln_i32_type, None);
    }
}

/// 代码生成错误
#[derive(Debug)]
pub struct CodegenError {
    pub message: String,
}

impl From<inkwell::builder::BuilderError> for CodegenError {
    fn from(e: inkwell::builder::BuilderError) -> Self {
        CodegenError { message: e.to_string() }
    }
}

impl<'ctx> CodeGenerator<'ctx> {
    /// 生成程序
    pub fn generate_program(&mut self, program: &Program) -> Result<(), CodegenError> {
        // 生成 struct 定义
        for struct_def in &program.structs {
            self.generate_struct_definition(struct_def)?;
        }

        // 生成函数声明
        for func in &program.functions {
            self.generate_function_declaration(func)?;
        }

        // 生成函数体
        for func in &program.functions {
            self.generate_function(func)?;
        }

        Ok(())
    }

    /// 生成 struct 定义
    fn generate_struct_definition(
        &mut self,
        struct_def: &StructDefinition,
    ) -> Result<(), CodegenError> {
        let field_types: Vec<inkwell::types::BasicTypeEnum> =
            struct_def.fields.iter().map(|field| self.map_type(&field.field_type)).collect();

        let struct_type = self.context.opaque_struct_type(&struct_def.name);
        struct_type.set_body(&field_types, false);

        self.struct_types.insert(struct_def.name.clone(), struct_type);
        Ok(())
    }

    /// 生成函数声明
    fn generate_function_declaration(&mut self, func: &Function) -> Result<(), CodegenError> {
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> =
            func.parameters.iter().map(|p| self.map_type(&p.type_annotation).into()).collect();

        let return_type = self.map_type(&func.return_type);
        let fn_type = return_type.fn_type(&param_types, false);

        self.module.add_function(&func.name, fn_type, None);
        Ok(())
    }

    /// 生成函数体
    fn generate_function(&mut self, func: &Function) -> Result<(), CodegenError> {
        let fn_value = self
            .module
            .get_function(&func.name)
            .ok_or_else(|| CodegenError { message: format!("Function {} not found", func.name) })?;

        // 创建 entry 块
        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder().position_at_end(entry);

        // 清空局部变量
        self.clear_variables();

        // 设置当前函数
        self.set_function(fn_value);

        // 为参数创建 alloca
        for (i, param) in func.parameters.iter().enumerate() {
            let param_value = fn_value.get_nth_param(i as u32).unwrap();
            let alloca = self.builder().build_alloca(param_value.get_type(), &param.name)?;
            // 将参数值存储到 alloca - 需要先转换为 BasicValue
            let basic_value = param_value;
            self.builder().build_store(alloca, basic_value)?;
            self.add_variable(param.name.clone(), alloca);
            // 保存参数类型
            self.set_variable_type(&param.name, param_value.get_type());
        }

        // 生成函数体
        self.generate_statement(&func.body)?;

        // 如果函数没有返回语句，添加一个默认返回
        let current_block = self.builder().get_insert_block().unwrap();
        if current_block.get_terminator().is_none() {
            // 统一返回 i32
            let default = self.context.i32_type().const_int(0, false);
            self.builder().build_return(Some(&default))?;
        }

        Ok(())
    }

    /// 生成语句
    pub fn generate_statement(&mut self, stmt: &Statement) -> Result<(), CodegenError> {
        match stmt {
            Statement::Block(stmts, _) => {
                for s in stmts {
                    self.generate_statement(s)?;
                }
            },
            Statement::VariableDeclaration {
                name,
                type_annotation,
                initializer,
                mutable: _,
                span: _,
            } => {
                // 首先检查是否有类型注解
                let type_from_annotation = if let Some(Type::Struct(struct_name)) = type_annotation
                {
                    Some(struct_name.clone())
                } else {
                    None
                };

                // 尝试从初始化表达式中推断 struct 类型
                let type_from_init = if let Some(Expression::StructLiteral {
                    name: struct_name,
                    ..
                }) = initializer
                {
                    Some(struct_name.clone())
                } else {
                    None
                };

                // 优先使用类型注解，如果没有则使用推断的类型
                let struct_name = type_from_annotation.or(type_from_init);

                if let Some(struct_name) = struct_name {
                    // 对于 struct 类型，需要获取 struct 类型并分配内存
                    // 创建一个虚拟的字段列表用于获取 struct 类型
                    let dummy_fields = vec![];
                    let struct_type =
                        self.get_or_create_struct_type(&struct_name, &dummy_fields)?;

                    // 创建局部变量（指针）
                    let alloca = self.builder().build_alloca(struct_type, name)?;

                    // 初始化
                    if let Some(init) = initializer {
                        let value = self.generate_expression(init)?;
                        // 存储 struct 值
                        self.builder().build_store(alloca, value)?;
                    }

                    self.add_variable(name.clone(), alloca);
                    // 保存结构体类型信息
                    self.set_variable_type(name, struct_type.into());
                } else {
                    // 确定类型
                    let ty = type_annotation
                        .as_ref()
                        .map(|t| self.map_type(t))
                        .unwrap_or_else(|| self.context.i32_type().into());

                    // 创建局部变量
                    let alloca = self.builder().build_alloca(ty, name)?;

                    // 初始化
                    if let Some(init) = initializer {
                        let value = self.generate_expression(init)?;
                        // 转换为 BasicValueEnum 存储
                        let basic_value = value;
                        self.builder().build_store(alloca, basic_value)?;
                    }

                    self.add_variable(name.clone(), alloca);
                    // 保存类型信息
                    self.set_variable_type(name, ty);
                }
            },
            Statement::Assignment { target, value, span: _ } => {
                if let Expression::Identifier(name, _) = target {
                    if let Some(ptr) = self.get_variable(name) {
                        let value = self.generate_expression(value)?;
                        let basic_value = value;
                        self.builder().build_store(ptr, basic_value)?;
                    } else {
                        return Err(CodegenError {
                            message: format!("Variable {} not found", name),
                        });
                    }
                }
            },
            Statement::If { condition, then_branch, else_branch, span: _ } => {
                let condition_value = self.generate_expression(condition)?;

                // 将条件值转换为 i1 进行条件分支
                let condition_i1 = match condition_value {
                    BasicValueEnum::IntValue(v) => {
                        if v.get_type().get_bit_width() == 1 {
                            // 已经是 i1 类型，直接使用
                            v
                        } else {
                            // i32 或其他整数类型：与0比较
                            let zero = v.get_type().const_int(0, false);
                            self.builder().build_int_compare(
                                inkwell::IntPredicate::NE,
                                v,
                                zero,
                                "condition_cmp",
                            )?
                        }
                    },
                    _ => {
                        // 其他类型，假设为非零
                        let i1_type = self.context.bool_type();
                        let one = i1_type.const_int(1, false);
                        let zero = i1_type.const_int(0, false);
                        self.builder().build_int_compare(
                            inkwell::IntPredicate::NE,
                            one,
                            zero,
                            "cond_to_bool",
                        )?
                    },
                };

                let function = self.current_function.unwrap();
                let then_block = self.context.append_basic_block(function, "then");
                let else_block = self.context.append_basic_block(function, "else");
                let merge_block = self.context.append_basic_block(function, "merge");

                self.builder().build_conditional_branch(condition_i1, then_block, else_block)?;

                // then 分支
                self.builder().position_at_end(then_block);
                self.generate_statement(then_branch)?;
                if self.builder().get_insert_block().unwrap().get_terminator().is_none() {
                    self.builder().build_unconditional_branch(merge_block)?;
                }

                // else 分支
                self.builder().position_at_end(else_block);
                if let Some(else_b) = else_branch {
                    self.generate_statement(else_b)?;
                }
                if self.builder().get_insert_block().unwrap().get_terminator().is_none() {
                    self.builder().build_unconditional_branch(merge_block)?;
                }

                // merge 块
                self.builder().position_at_end(merge_block);
            },
            Statement::While { condition, body, span: _ } => {
                let function = self.current_function.unwrap();
                let cond_block = self.context.append_basic_block(function, "while_cond");
                let body_block = self.context.append_basic_block(function, "while_body");
                let end_block = self.context.append_basic_block(function, "while_end");

                self.builder().build_unconditional_branch(cond_block)?;

                // 条件块
                self.builder().position_at_end(cond_block);
                let condition_value = self.generate_expression(condition)?;

                let condition_i1 = match condition_value {
                    BasicValueEnum::IntValue(v) => {
                        if v.get_type().get_bit_width() == 1 {
                            v
                        } else {
                            let zero = v.get_type().const_int(0, false);
                            self.builder().build_int_compare(
                                inkwell::IntPredicate::NE,
                                v,
                                zero,
                                "condition_cmp",
                            )?
                        }
                    },
                    _ => {
                        let i1_type = self.context.bool_type();
                        let one = i1_type.const_int(1, false);
                        let zero = i1_type.const_int(0, false);
                        self.builder().build_int_compare(
                            inkwell::IntPredicate::NE,
                            one,
                            zero,
                            "cond_to_bool",
                        )?
                    },
                };
                self.builder().build_conditional_branch(condition_i1, body_block, end_block)?;

                // 循环体
                self.builder().position_at_end(body_block);
                self.generate_statement(body)?;
                if self.builder().get_insert_block().unwrap().get_terminator().is_none() {
                    self.builder().build_unconditional_branch(cond_block)?;
                }

                // 结束块
                self.builder().position_at_end(end_block);
            },
            Statement::Return(value, _span) => {
                if let Some(v) = value {
                    let ret_value = self.generate_expression(v)?;
                    self.builder().build_return(Some(&ret_value))?;
                } else {
                    self.builder().build_return(None)?;
                }
            },
            Statement::Break(_span) => {},
            Statement::Continue(_span) => {},
            Statement::ExpressionStatement(expr) => {
                self.generate_expression(expr)?;
            },
            Statement::Empty(_span) => {},
            Statement::For { initializer, condition, update, body, span: _ } => {
                self.generate_statement(initializer)?;

                let function = self.current_function.unwrap();
                let cond_block = self.context.append_basic_block(function, "for_cond");
                let body_block = self.context.append_basic_block(function, "for_body");
                let update_block = self.context.append_basic_block(function, "for_update");
                let end_block = self.context.append_basic_block(function, "for_end");

                self.builder().build_unconditional_branch(cond_block)?;

                // 条件块
                self.builder().position_at_end(cond_block);
                if let Some(cond) = condition {
                    let condition_value = self.generate_expression(cond)?;

                    let condition_i1 = match condition_value {
                        BasicValueEnum::IntValue(v) => {
                            if v.get_type().get_bit_width() == 1 {
                                v
                            } else {
                                let zero = v.get_type().const_int(0, false);
                                self.builder().build_int_compare(
                                    inkwell::IntPredicate::NE,
                                    v,
                                    zero,
                                    "condition_cmp",
                                )?
                            }
                        },
                        _ => {
                            let i1_type = self.context.bool_type();
                            let one = i1_type.const_int(1, false);
                            let zero = i1_type.const_int(0, false);
                            self.builder().build_int_compare(
                                inkwell::IntPredicate::NE,
                                one,
                                zero,
                                "cond_to_bool",
                            )?
                        },
                    };
                    self.builder().build_conditional_branch(condition_i1, body_block, end_block)?;
                } else {
                    self.builder().build_unconditional_branch(body_block)?;
                }

                // 循环体
                self.builder().position_at_end(body_block);
                self.generate_statement(body)?;
                if self.builder().get_insert_block().unwrap().get_terminator().is_none() {
                    self.builder().build_unconditional_branch(update_block)?;
                }

                // 更新块
                self.builder().position_at_end(update_block);
                if let Some(upd) = update {
                    self.generate_expression(upd)?;
                }
                if self.builder().get_insert_block().unwrap().get_terminator().is_none() {
                    self.builder().build_unconditional_branch(cond_block)?;
                }

                // 结束块
                self.builder().position_at_end(end_block);
            },
            Statement::Switch { value, arms, span: _ } => {
                self.generate_switch(value.as_ref(), arms)?;
            },
        }

        Ok(())
    }

    /// 生成表达式
    pub fn generate_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        match expr {
            Expression::Number(n, _span) => {
                let i32_type = self.context.i32_type();
                Ok(i32_type.const_int(*n as u64, false).into())
            },
            Expression::Float(f, _span) => {
                let f64_type = self.context.f64_type();
                Ok(f64_type.const_float(*f).into())
            },
            Expression::String(s, _span) => {
                let str_with_null = format!("{}\0", s);
                let llvm_str = self.context.const_string(str_with_null.as_bytes(), true);
                let global = self.module.add_global(llvm_str.get_type(), None, "str");
                global.set_initializer(&llvm_str);

                let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
                let ptr = self.builder().build_bit_cast(global, i8_ptr_type, "str_ptr")?;
                Ok(ptr)
            },
            Expression::Boolean(b, _span) => {
                let i32_type = self.context.i32_type();
                Ok(i32_type.const_int(if *b { 1 } else { 0 }, false).into())
            },
            Expression::Identifier(name, _span) => {
                if let Some(ptr) = self.get_variable(name) {
                    // 根据保存的类型信息加载变量
                    if let Some(var_type) = self.get_variable_type(name) {
                        let value = self.builder().build_load(var_type, ptr, name)?;
                        Ok(value)
                    } else {
                        // 默认使用 i32 类型（兼容旧代码）
                        let i32_type = self.context.i32_type();
                        let value = self.builder().build_load(i32_type, ptr, name)?;
                        Ok(value)
                    }
                } else {
                    Err(CodegenError { message: format!("Variable {} not found", name) })
                }
            },
            Expression::Binary { op, left, right, span: _ } => {
                let lhs = self.generate_expression(left)?;
                let rhs = self.generate_expression(right)?;

                let result = match op {
                    BinaryOp::Add => {
                        if lhs.is_int_value() {
                            self.builder()
                                .build_int_add(lhs.into_int_value(), rhs.into_int_value(), "add")?
                                .into()
                        } else {
                            self.builder()
                                .build_float_add(
                                    lhs.into_float_value(),
                                    rhs.into_float_value(),
                                    "fadd",
                                )?
                                .into()
                        }
                    },
                    BinaryOp::Subtract => {
                        if lhs.is_int_value() {
                            self.builder()
                                .build_int_sub(lhs.into_int_value(), rhs.into_int_value(), "sub")?
                                .into()
                        } else {
                            self.builder()
                                .build_float_sub(
                                    lhs.into_float_value(),
                                    rhs.into_float_value(),
                                    "fsub",
                                )?
                                .into()
                        }
                    },
                    BinaryOp::Multiply => {
                        if lhs.is_int_value() {
                            self.builder()
                                .build_int_mul(lhs.into_int_value(), rhs.into_int_value(), "mul")?
                                .into()
                        } else {
                            self.builder()
                                .build_float_mul(
                                    lhs.into_float_value(),
                                    rhs.into_float_value(),
                                    "fmul",
                                )?
                                .into()
                        }
                    },
                    BinaryOp::Divide => {
                        if lhs.is_int_value() {
                            self.builder()
                                .build_int_signed_div(
                                    lhs.into_int_value(),
                                    rhs.into_int_value(),
                                    "div",
                                )?
                                .into()
                        } else {
                            self.builder()
                                .build_float_div(
                                    lhs.into_float_value(),
                                    rhs.into_float_value(),
                                    "fdiv",
                                )?
                                .into()
                        }
                    },
                    BinaryOp::Modulo => self
                        .builder()
                        .build_int_signed_rem(lhs.into_int_value(), rhs.into_int_value(), "rem")?
                        .into(),
                    BinaryOp::Equals => {
                        if lhs.is_int_value() {
                            self.builder()
                                .build_int_compare(
                                    inkwell::IntPredicate::EQ,
                                    lhs.into_int_value(),
                                    rhs.into_int_value(),
                                    "eq",
                                )?
                                .into()
                        } else {
                            self.builder()
                                .build_float_compare(
                                    inkwell::FloatPredicate::OEQ,
                                    lhs.into_float_value(),
                                    rhs.into_float_value(),
                                    "eq",
                                )?
                                .into()
                        }
                    },
                    BinaryOp::NotEquals => {
                        if lhs.is_int_value() {
                            self.builder()
                                .build_int_compare(
                                    inkwell::IntPredicate::NE,
                                    lhs.into_int_value(),
                                    rhs.into_int_value(),
                                    "ne",
                                )?
                                .into()
                        } else {
                            self.builder()
                                .build_float_compare(
                                    inkwell::FloatPredicate::ONE,
                                    lhs.into_float_value(),
                                    rhs.into_float_value(),
                                    "ne",
                                )?
                                .into()
                        }
                    },
                    BinaryOp::LessThan => {
                        if lhs.is_int_value() {
                            self.builder()
                                .build_int_compare(
                                    inkwell::IntPredicate::SLT,
                                    lhs.into_int_value(),
                                    rhs.into_int_value(),
                                    "lt",
                                )?
                                .into()
                        } else {
                            self.builder()
                                .build_float_compare(
                                    inkwell::FloatPredicate::OLT,
                                    lhs.into_float_value(),
                                    rhs.into_float_value(),
                                    "lt",
                                )?
                                .into()
                        }
                    },
                    BinaryOp::LessThanOrEqual => {
                        if lhs.is_int_value() {
                            self.builder()
                                .build_int_compare(
                                    inkwell::IntPredicate::SLE,
                                    lhs.into_int_value(),
                                    rhs.into_int_value(),
                                    "le",
                                )?
                                .into()
                        } else {
                            self.builder()
                                .build_float_compare(
                                    inkwell::FloatPredicate::OLE,
                                    lhs.into_float_value(),
                                    rhs.into_float_value(),
                                    "le",
                                )?
                                .into()
                        }
                    },
                    BinaryOp::GreaterThan => {
                        if lhs.is_int_value() {
                            self.builder()
                                .build_int_compare(
                                    inkwell::IntPredicate::SGT,
                                    lhs.into_int_value(),
                                    rhs.into_int_value(),
                                    "gt",
                                )?
                                .into()
                        } else {
                            self.builder()
                                .build_float_compare(
                                    inkwell::FloatPredicate::OGT,
                                    lhs.into_float_value(),
                                    rhs.into_float_value(),
                                    "gt",
                                )?
                                .into()
                        }
                    },
                    BinaryOp::GreaterThanOrEqual => {
                        if lhs.is_int_value() {
                            self.builder()
                                .build_int_compare(
                                    inkwell::IntPredicate::SGE,
                                    lhs.into_int_value(),
                                    rhs.into_int_value(),
                                    "ge",
                                )?
                                .into()
                        } else {
                            self.builder()
                                .build_float_compare(
                                    inkwell::FloatPredicate::OGE,
                                    lhs.into_float_value(),
                                    rhs.into_float_value(),
                                    "ge",
                                )?
                                .into()
                        }
                    },
                    BinaryOp::LogicalAnd => {
                        let i32_type = self.context.i32_type();
                        let zero = i32_type.const_int(0, false);
                        let lhs_nonzero = self.builder().build_int_compare(
                            inkwell::IntPredicate::NE,
                            lhs.into_int_value(),
                            zero,
                            "lhs_nz",
                        )?;
                        let rhs_nonzero = self.builder().build_int_compare(
                            inkwell::IntPredicate::NE,
                            rhs.into_int_value(),
                            zero,
                            "rhs_nz",
                        )?;
                        self.builder().build_and(lhs_nonzero, rhs_nonzero, "and")?.into()
                    },
                    BinaryOp::LogicalOr => {
                        let i32_type = self.context.i32_type();
                        let zero = i32_type.const_int(0, false);
                        let lhs_nonzero = self.builder().build_int_compare(
                            inkwell::IntPredicate::NE,
                            lhs.into_int_value(),
                            zero,
                            "lhs_nz",
                        )?;
                        let rhs_nonzero = self.builder().build_int_compare(
                            inkwell::IntPredicate::NE,
                            rhs.into_int_value(),
                            zero,
                            "rhs_nz",
                        )?;
                        self.builder().build_or(lhs_nonzero, rhs_nonzero, "or")?.into()
                    },
                    _ => {
                        return Err(CodegenError {
                            message: format!("Unsupported binary operator: {:?}", op),
                        });
                    },
                };

                Ok(result)
            },
            Expression::Unary { op, operand, span: _ } => {
                let value = self.generate_expression(operand)?;
                match op {
                    UnaryOp::Negate => {
                        if value.is_int_value() {
                            let zero = self.context.i32_type().const_int(0, false);
                            Ok(self
                                .builder()
                                .build_int_sub(zero, value.into_int_value(), "neg")?
                                .into())
                        } else {
                            let zero = self.context.f64_type().const_float(0.0);
                            Ok(self
                                .builder()
                                .build_float_sub(zero, value.into_float_value(), "fneg")?
                                .into())
                        }
                    },
                    UnaryOp::LogicalNot => {
                        let i32_type = self.context.i32_type();
                        let zero = i32_type.const_int(0, false);
                        let nonzero = self.builder().build_int_compare(
                            inkwell::IntPredicate::EQ,
                            value.into_int_value(),
                            zero,
                            "is_zero",
                        )?;
                        Ok(nonzero.into())
                    },
                    UnaryOp::BitNot => {
                        // 位取反
                        if value.is_int_value() {
                            let result = self.builder().build_not(value.into_int_value(), "not")?;
                            Ok(result.into())
                        } else {
                            Err(CodegenError {
                                message: "BitNot requires integer operand".to_string(),
                            })
                        }
                    },
                    UnaryOp::AddressOf => {
                        // 取地址 &x
                        // 需要特殊处理：因为 generate_expression 会返回加载后的值
                        // 所以我们直接在 operand 是标识符时获取其指针
                        if let Expression::Identifier(name, _) = operand.as_ref() {
                            if let Some(ptr) = self.get_variable(name) {
                                Ok(ptr.into())
                            } else {
                                Err(CodegenError {
                                    message: format!("Variable {} not found for address-of", name),
                                })
                            }
                        } else {
                            Err(CodegenError {
                                message: "AddressOf only supports variables".to_string(),
                            })
                        }
                    },
                    UnaryOp::Dereference => {
                        // 解引用 *x
                        // 需要特殊处理：因为 generate_expression 会返回加载后的值
                        // 所以我们直接在 operand 是标识符时获取其指针
                        if let Expression::Identifier(name, _) = operand.as_ref() {
                            if let Some(ptr) = self.get_variable(name) {
                                // 加载指针指向的值
                                let i32_type = self.context.i32_type();
                                let result = self.builder().build_load(i32_type, ptr, "deref")?;
                                Ok(result)
                            } else {
                                Err(CodegenError {
                                    message: format!("Variable {} not found for dereference", name),
                                })
                            }
                        } else {
                            Err(CodegenError {
                                message: "Dereference only supports variables".to_string(),
                            })
                        }
                    },
                }
            },
            Expression::Call { callee, arguments, span: _ } => {
                let callee_name = match callee.as_ref() {
                    Expression::Identifier(name, _) => name.clone(),
                    _ => {
                        return Err(CodegenError {
                            message: "Only function name calls supported".to_string(),
                        });
                    },
                };

                // 处理内置函数
                if callee_name == "println" || callee_name == "console.log" {
                    // 简化处理: println/console.log 只支持一个字符串参数
                    let puts_fn = self.module.get_function("puts").ok_or_else(|| CodegenError {
                        message: "puts function not found".to_string(),
                    })?;

                    let i8_ptr_type = self.context.ptr_type(AddressSpace::default());

                    if arguments.is_empty() {
                        // 打印空行
                        let empty_str = self.context.const_string(b"\n\0", true);
                        let global =
                            self.module.add_global(empty_str.get_type(), None, "empty_str");
                        global.set_initializer(&empty_str);
                        let ptr =
                            self.builder().build_bit_cast(global, i8_ptr_type, "empty_ptr")?;
                        self.builder().build_call(puts_fn, &[ptr.into()], "puts_call")?;
                    } else {
                        // 处理第一个参数
                        let arg = &arguments[0];
                        let arg_value = self.generate_expression(arg)?;

                        // 转换为 i8* 指针
                        let ptr =
                            self.builder().build_bit_cast(arg_value, i8_ptr_type, "arg_ptr")?;
                        self.builder().build_call(puts_fn, &[ptr.into()], "puts_call")?;

                        // 添加换行
                        let newline_str = self.context.const_string(b"\n\0", true);
                        let newline_global =
                            self.module.add_global(newline_str.get_type(), None, "newline_str");
                        newline_global.set_initializer(&newline_str);
                        let newline_ptr = self.builder().build_bit_cast(
                            newline_global,
                            i8_ptr_type,
                            "newline_ptr",
                        )?;
                        self.builder().build_call(
                            puts_fn,
                            &[newline_ptr.into()],
                            "puts_newline",
                        )?;
                    }

                    return Ok(self.context.i32_type().const_int(0, false).into());
                }

                // 处理 readln 函数
                if callee_name == "readln" || callee_name == "readln_i32" {
                    let i32_type = self.context.i32_type();
                    let i8_ptr_type = self.context.ptr_type(AddressSpace::default());

                    // 分配一个 i32 变量来存储输入
                    let input_var = self.builder().build_alloca(i32_type, "input_var")?;

                    // 创建格式字符串 "%d"
                    let format_str = self.context.const_string(b"%d\0", true);
                    let format_global =
                        self.module.add_global(format_str.get_type(), None, "readln_format");
                    format_global.set_initializer(&format_str);
                    let format_ptr =
                        self.builder().build_bit_cast(format_global, i8_ptr_type, "format_ptr")?;

                    // 调用 scanf
                    let scanf_fn = self.module.get_function("scanf").ok_or_else(|| {
                        CodegenError { message: "scanf function not found".to_string() }
                    })?;
                    self.builder().build_call(
                        scanf_fn,
                        &[format_ptr.into(), input_var.into()],
                        "scanf_call",
                    )?;

                    // 加载输入的值
                    let result = self.builder().build_load(i32_type, input_var, "readln_result")?;
                    return Ok(result);
                }

                let fn_value = self.module.get_function(&callee_name).ok_or_else(|| {
                    CodegenError { message: format!("Function {} not found", callee_name) }
                })?;

                let mut args_values: Vec<inkwell::values::BasicMetadataValueEnum> = Vec::new();
                for arg in arguments {
                    let arg_val = self.generate_expression(arg)?;
                    args_values.push(arg_val.into());
                }

                let call_result = self.builder().build_call(fn_value, &args_values, "call")?;

                // 获取函数返回类型
                let fn_return_type = fn_value.get_type().get_return_type();
                let i32_type = self.context.i32_type();
                if let Some(_return_type) = fn_return_type {
                    // 如果有返回值，尝试获取返回值
                    let value_kind = call_result.try_as_basic_value();
                    if let inkwell::values::ValueKind::Basic(basic_value) = value_kind {
                        return Ok(basic_value);
                    }
                }
                // 默认返回 0
                Ok(i32_type.const_int(0, false).into())
            },
            Expression::Index { array, index, span: _ } => {
                // 生成数组和索引值
                let array_value = self.generate_expression(array)?;
                let index_value = self.generate_expression(index)?;

                // 获取数组的指针值
                let array_ptr = array_value.into_pointer_value();

                // 将索引转换为 i32
                let index_i32 = self.coerce_to_i32(index_value)?;

                // 使用 GEP 获取元素指针（假设数组元素是 i32）
                // 注意：如果 array_ptr 已经指向数组数据，直接用 index_i32
                let i32_type = self.context.i32_type();
                let element_ptr = unsafe {
                    self.builder().build_in_bounds_gep(
                        i32_type,
                        array_ptr,
                        &[index_i32],
                        "array_element_ptr",
                    )
                }?;

                // 加载元素值
                let element = self.builder().build_load(i32_type, element_ptr, "array_element")?;
                Ok(element)
            },
            Expression::StructLiteral { name, fields, span: _ } => {
                // 获取或创建 struct 类型
                let struct_type = self.get_or_create_struct_type(name, fields)?;
                // 分配内存
                let alloca = self.builder().build_alloca(struct_type, "struct_alloca")?;
                // 预先创建索引常量避免借用问题
                let zero = self.context.i32_type().const_int(0, false);
                // 初始化字段
                for (i, (field_name, field_expr)) in fields.iter().enumerate() {
                    let field_value = self.generate_expression(field_expr)?;
                    // 获取字段指针
                    let idx = self.context.i32_type().const_int(i as u64, false);
                    let field_ptr = unsafe {
                        self.builder().build_in_bounds_gep(
                            struct_type,
                            alloca,
                            &[zero, idx],
                            field_name,
                        )?
                    };
                    self.builder().build_store(field_ptr, field_value)?;
                }
                // 加载整个 struct 作为返回值
                let loaded = self.builder().build_load(struct_type, alloca, "struct_load")?;
                Ok(loaded)
            },
            Expression::Member { object, member, span: _ } => {
                // 检查对象是否是标识符
                if let Expression::Identifier(var_name, _) = object.as_ref() {
                    // 如果是标识符，获取变量的指针
                    if let Some(ptr) = self.get_variable(var_name) {
                        // 获取变量类型
                        let _var_type_opt = self.get_variable_type(var_name);

                        // 预先创建索引常量
                        let i32_type = self.context.i32_type();
                        let zero = i32_type.const_int(0, false);

                        // 根据变量类型确定字段索引和类型
                        let field_index = match member.as_str() {
                            "x" => 0,
                            "y" => 1,
                            "z" => 2,
                            _ => 0,
                        };
                        let idx = i32_type.const_int(field_index as u64, false);

                        // 尝试获取 struct 类型（目前未使用，保留用于未来实现）
                        let _struct_type_opt: Option<inkwell::types::StructType<'ctx>> = None;

                        // 查找 struct 类型
                        let struct_type = self.find_struct_type_from_variable(var_name);

                        if let Some(struct_type) = struct_type {
                            // 使用 struct 类型进行 GEP 获取字段地址
                            let field_ptr = unsafe {
                                self.builder().build_in_bounds_gep(
                                    struct_type,
                                    ptr,
                                    &[zero, idx],
                                    member,
                                )?
                            };

                            // 获取字段的实际类型并加载
                            let field_types = struct_type.get_field_types();
                            let field_llvm_type = field_types[field_index];
                            let field_value =
                                self.builder().build_load(field_llvm_type, field_ptr, member)?;
                            return Ok(field_value);
                        }

                        // 如果没有找到 struct 类型，尝试使用 i32（兼容旧代码）
                        let field_ptr = unsafe {
                            self.builder().build_in_bounds_gep(
                                i32_type,
                                ptr,
                                &[zero, idx],
                                member,
                            )?
                        };

                        // 加载字段值
                        let field_value = self.builder().build_load(i32_type, field_ptr, member)?;
                        return Ok(field_value);
                    }
                }

                // 对于非标识符的对象，生成表达式
                let obj_value = self.generate_expression(object)?;

                // 检查对象的类型
                if obj_value.is_pointer_value() {
                    // 如果是指针，假设是 struct 指针
                    // 简化处理：假设字段在 struct 中的偏移量为 i32
                    let field_index = if member == "x" {
                        0
                    } else if member == "y" {
                        1
                    } else if member == "z" {
                        2
                    } else {
                        0
                    };

                    // 预先创建索引常量
                    let i32_type = self.context.i32_type();
                    let zero = i32_type.const_int(0, false);
                    let idx = i32_type.const_int(field_index as u64, false);

                    // 使用 struct 作为基本类型进行 GEP
                    let field_ptr = unsafe {
                        self.builder().build_in_bounds_gep(
                            i32_type,
                            obj_value.into_pointer_value(),
                            &[zero, idx],
                            member,
                        )?
                    };

                    // 加载字段值
                    let field_value = self.builder().build_load(i32_type, field_ptr, member)?;
                    Ok(field_value)
                } else if obj_value.is_struct_value() {
                    // 如果是 struct 值，使用 extractvalue
                    let field_index = if member == "x" {
                        0
                    } else if member == "y" {
                        1
                    } else if member == "z" {
                        2
                    } else {
                        0
                    };

                    // 使用 extractvalue 提取字段
                    let field_value = self.builder().build_extract_value(
                        obj_value.into_struct_value(),
                        field_index,
                        member,
                    )?;
                    Ok(field_value)
                } else {
                    Err(CodegenError {
                        message: format!(
                            "Member access on unsupported type: {:?}",
                            obj_value.get_type()
                        ),
                    })
                }
            },
            Expression::Assignment { target, value, span: _ } => {
                // 简化处理：支持 identifier = expr 形式的赋值
                let value = self.generate_expression(value)?;
                if let Expression::Identifier(name, _) = target.as_ref() {
                    if let Some(ptr) = self.get_variable(name) {
                        self.builder().build_store(ptr, value)?;
                    }
                }
                Ok(value)
            },
            Expression::FunctionExpression { .. } => {
                // 函数表达式暂不支持作为表达式值
                Err(CodegenError {
                    message: "Function expressions are not supported as values".to_string(),
                })
            },
            Expression::ArrayLiteral { elements, span: _ } => {
                // 数组字面量: 为每个元素分配内存
                let i32_type = self.context.i32_type();
                let _element_count = elements.len();

                // 分配数组内存（使用 alloca）
                let array_alloca = self.builder().build_alloca(i32_type, "array_literal")?;

                // 存储每个元素
                for (i, elem) in elements.iter().enumerate() {
                    let elem_value = self.generate_expression(elem)?;
                    let idx = i32_type.const_int(i as u64, false);
                    let elem_ptr = unsafe {
                        self.builder().build_in_bounds_gep(
                            i32_type,
                            array_alloca,
                            &[idx],
                            "array_element_ptr",
                        )?
                    };
                    self.builder().build_store(elem_ptr, elem_value)?;
                }

                // 返回数组指针
                Ok(array_alloca.into())
            },
            Expression::Ternary { condition, then_expr, else_expr, span: _ } => {
                // 三元表达式: condition ? then_expr : else_expr
                let i32_type = self.context.i32_type();

                // 生成条件值
                let cond_value = self.generate_expression(condition)?;

                // 将条件转换为 i1 进行分支判断
                let cond_i1 = match cond_value {
                    inkwell::values::BasicValueEnum::IntValue(v) => {
                        // 如果是 i1 类型，直接使用
                        if v.get_type().get_bit_width() == 1 {
                            v
                        } else {
                            // 如果是 i32，转换为 i1 (非零为 true)
                            let zero = i32_type.const_int(0, false);
                            self.builder().build_int_compare(
                                inkwell::IntPredicate::NE,
                                v,
                                zero,
                                "cond",
                            )?
                        }
                    },
                    _ => {
                        return Err(CodegenError {
                            message: "Ternary condition must be integer".to_string(),
                        });
                    },
                };

                // 创建基本块
                let current_function =
                    self.builder().get_insert_block().and_then(|b| b.get_parent()).ok_or_else(
                        || CodegenError { message: "Could not get current function".to_string() },
                    )?;

                let then_block = self.context.append_basic_block(current_function, "ternary_then");
                let else_block = self.context.append_basic_block(current_function, "ternary_else");
                let merge_block =
                    self.context.append_basic_block(current_function, "ternary_merge");

                // 条件分支
                self.builder().build_conditional_branch(cond_i1, then_block, else_block)?;

                // 生成 then 分支
                self.builder().position_at_end(then_block);
                let then_value = self.generate_expression(then_expr)?;
                self.builder().build_unconditional_branch(merge_block)?;

                // 生成 else 分支
                self.builder().position_at_end(else_block);
                let else_value = self.generate_expression(else_expr)?;
                self.builder().build_unconditional_branch(merge_block)?;

                // 在 merge 块中创建 phi 节点
                self.builder().position_at_end(merge_block);
                let phi = self.builder().build_phi(i32_type, "ternary_result")?;
                phi.add_incoming(&[(&then_value, then_block), (&else_value, else_block)]);

                Ok(phi.as_basic_value())
            },
        }
    }

    /// 获取或创建 struct 类型
    fn get_or_create_struct_type(
        &mut self,
        name: &str,
        fields: &[(String, Expression)],
    ) -> Result<inkwell::types::StructType<'ctx>, CodegenError> {
        // 检查是否已存在
        if let Some(struct_type) = self.struct_types.get(name) {
            return Ok(*struct_type);
        }

        // 创建 struct 类型
        let field_types: Vec<inkwell::types::BasicTypeEnum> = fields
            .iter()
            .map(|(_, _expr)| {
                // 简化处理：假设所有字段都是 i32
                self.context.i32_type().into()
            })
            .collect();

        let struct_type = self.context.opaque_struct_type(name);
        struct_type.set_body(&field_types, false);

        self.struct_types.insert(name.to_string(), struct_type);
        Ok(struct_type)
    }

    /// 从变量查找 struct 类型
    /// 遍历所有 struct 类型，找到变量对应的 struct 类型
    fn find_struct_type_from_variable(
        &self,
        var_name: &str,
    ) -> Option<inkwell::types::StructType<'ctx>> {
        // 检查变量类型映射
        if let Some(var_type) = self.variable_types.get(var_name) {
            if var_type.is_struct_type() {
                let struct_type = var_type.into_struct_type();
                // 尝试在 struct_types 中查找匹配的类型
                for st in self.struct_types.values() {
                    if *st == struct_type {
                        return Some(*st);
                    }
                }
            }
        }
        None
    }

    /// 将 Nexa 类型映射到 LLVM 类型 (TypeScript 风格)
    fn map_type(&self, ty: &Type) -> BasicTypeEnum<'ctx> {
        match ty {
            Type::Number => self.context.i32_type().into(),
            Type::Boolean => self.context.i32_type().into(),
            Type::String => self.context.ptr_type(AddressSpace::default()).into(),
            Type::Void => {
                // Void 类型作为函数返回类型时不映射到 BasicTypeEnum
                // 这里返回 i32 作为占位符
                self.context.i32_type().into()
            },
            Type::Undefined => self.context.i32_type().into(),
            Type::Null => self.context.ptr_type(AddressSpace::default()).into(),
            Type::Any => self.context.ptr_type(AddressSpace::default()).into(),
            Type::Never => self.context.i32_type().into(),
            Type::Array(_) => self.context.ptr_type(AddressSpace::default()).into(),
            Type::Pointer(_) => self.context.ptr_type(AddressSpace::default()).into(),
            Type::Function(_, return_type) => self.map_type(return_type),
            Type::Struct(name) => {
                // 尝试查找已注册的 struct 类型
                if let Some(struct_type) = self.struct_types.get(name) {
                    (*struct_type).into()
                } else {
                    // 如果未找到，返回指针类型作为后备
                    self.context.ptr_type(AddressSpace::default()).into()
                }
            },
            Type::Object(_) => self.context.ptr_type(AddressSpace::default()).into(),
        }
    }

    /// 生成 switch 表达式
    /// 使用递归生成嵌套的 if-else 链
    fn generate_switch(
        &mut self,
        value: &Expression,
        arms: &[SwitchArm],
    ) -> Result<(), CodegenError> {
        let function = self.current_function.unwrap();
        let i64_type = self.context.i64_type();

        // 生成要匹配的值
        let switch_value = self.generate_expression(value)?;

        // 获取匹配的值并统一转换为 i64 进行比较
        let switch_int = if switch_value.is_int_value() {
            // i32 扩展为 i64
            self.builder().build_int_s_extend(
                switch_value.into_int_value(),
                i64_type,
                "switch_i64",
            )?
        } else if switch_value.is_pointer_value() {
            // 指针转换为 i64
            self.builder().build_ptr_to_int(
                switch_value.into_pointer_value(),
                i64_type,
                "ptr_to_int",
            )?
        } else {
            return Err(CodegenError {
                message: "Switch value must be integer or pointer type".to_string(),
            });
        };

        // 创建合并块
        let merge_block = self.context.append_basic_block(function, "switch_merge");

        // 递归生成 if-else 链
        self.generate_switch_arms(0, arms, switch_int, i64_type, merge_block)?;

        // 设置合并块位置
        self.builder().position_at_end(merge_block);

        Ok(())
    }

    /// 递归生成 switch 分支的 if-else 链
    fn generate_switch_arms(
        &mut self,
        idx: usize,
        arms: &[SwitchArm],
        switch_int: inkwell::values::IntValue<'ctx>,
        i64_type: inkwell::types::IntType<'ctx>,
        merge_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) -> Result<(), CodegenError> {
        if idx >= arms.len() {
            // 没有更多分支，跳转到 merge
            self.builder().build_unconditional_branch(merge_block)?;
            return Ok(());
        }

        let arm = &arms[idx];
        let function = self.current_function.unwrap();

        // 为这个分支创建 then 块
        let then_block = self.context.append_basic_block(function, "switch_then");

        // 确定 else 目标
        let else_block = if idx + 1 < arms.len() {
            // 还有更多分支，需要继续检查
            self.context.append_basic_block(function, "switch_else")
        } else {
            // 最后一个分支，跳转到 merge
            merge_block
        };

        // 生成条件比较
        let cond_i1: inkwell::values::IntValue<'ctx> = match &arm.pattern {
            SwitchPattern::Number(n) => {
                let n_int = i64_type.const_int(*n as u64, false);
                self.builder().build_int_compare(
                    inkwell::IntPredicate::EQ,
                    switch_int,
                    n_int,
                    "switch_cond",
                )?
            },
            SwitchPattern::Identifier(name) => {
                if let Some(ptr) = self.get_variable(name) {
                    let loaded = self.builder().build_load(ptr.get_type(), ptr, name)?;
                    let loaded_int = if loaded.is_int_value() {
                        loaded.into_int_value()
                    } else if loaded.is_pointer_value() {
                        self.builder().build_ptr_to_int(
                            loaded.into_pointer_value(),
                            i64_type,
                            "ptr_to_int",
                        )?
                    } else {
                        return Err(CodegenError {
                            message: "Switch identifier must be integer or pointer type"
                                .to_string(),
                        });
                    };
                    self.builder().build_int_compare(
                        inkwell::IntPredicate::EQ,
                        switch_int,
                        loaded_int,
                        "switch_cond",
                    )?
                } else {
                    // 变量不存在，条件为 false
                    i64_type.const_int(0, false)
                }
            },
            SwitchPattern::Wildcard | SwitchPattern::Default => {
                // 通配符/default：条件始终为 true
                // 创建全 1 的 i64 值然后转换为 i1
                let one = i64_type.const_int(1, false);
                let zero = i64_type.const_int(0, false);
                self.builder().build_int_compare(
                    inkwell::IntPredicate::NE,
                    one,
                    zero,
                    "default_cond",
                )?
            },
        };

        // 生成条件分支 (cond_i1 已经是 i1 类型)
        self.builder().build_conditional_branch(cond_i1, then_block, else_block)?;

        // 在 then 块中生成代码
        self.builder().position_at_end(then_block);
        self.generate_statement(&arm.body)?;
        if self.builder().get_insert_block().unwrap().get_terminator().is_none() {
            self.builder().build_unconditional_branch(merge_block)?;
        }

        // 继续处理 else 分支
        if idx + 1 < arms.len() {
            self.builder().position_at_end(else_block);
            self.generate_switch_arms(idx + 1, arms, switch_int, i64_type, merge_block)?;
        }

        Ok(())
    }
}
