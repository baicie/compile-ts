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
    /// 局部变量映射
    variables: HashMap<String, PointerValue<'ctx>>,
    /// 变量类型映射 (变量名 -> struct 类型名)
    variable_types: HashMap<String, String>,
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
    pub fn set_variable_type(&mut self, name: &str, type_name: &str) {
        self.variable_types.insert(name.to_string(), type_name.to_string());
    }

    /// 获取变量类型
    pub fn get_variable_type(&self, name: &str) -> Option<&String> {
        self.variable_types.get(name)
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

        // puts - C 标准库输出字符串
        let puts_type = self.context.i32_type().fn_type(&[i8_ptr.into()], false);
        self.module.add_function("puts", puts_type, None);

        // console.log (通过 puts 实现)
        let console_log_type = self.context.i32_type().fn_type(&[i8_ptr.into()], false);
        self.module.add_function("console.log", console_log_type, None);
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
                    // 设置变量类型
                    self.set_variable_type(name, &struct_name);
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
                    // 如果有类型注解，也保存类型信息
                    if let Some(Type::Struct(struct_name)) = type_annotation {
                        self.set_variable_type(name, struct_name);
                    }
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
            Statement::Match { value, arms, span: _ } => {
                self.generate_match(value.as_ref(), arms)?;
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
                    // 简化处理：假设变量是 i32 类型
                    let i32_type = self.context.i32_type();
                    let value = self.builder().build_load(i32_type, ptr, name)?;
                    Ok(value)
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
                    _ => Err(CodegenError { message: "Unsupported unary operator".to_string() }),
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

                let fn_value = self.module.get_function(&callee_name).ok_or_else(|| {
                    CodegenError { message: format!("Function {} not found", callee_name) }
                })?;

                let mut args_values: Vec<inkwell::values::BasicMetadataValueEnum> = Vec::new();
                for arg in arguments {
                    let arg_val = self.generate_expression(arg)?;
                    args_values.push(arg_val.into());
                }

                self.builder().build_call(fn_value, &args_values, "call")?;

                let i32_type = self.context.i32_type();
                Ok(i32_type.const_int(0, false).into())
            },
            Expression::Index { array: _, index: _, span: _ } => {
                Err(CodegenError { message: "Array index not implemented".to_string() })
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
                        // 尝试从变量类型映射中获取 struct 类型
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

                        // 尝试获取 struct 类型
                        let struct_type_opt =
                            if let Some(struct_name) = self.get_variable_type(var_name) {
                                self.struct_types.get(struct_name).copied()
                            } else {
                                None
                            };

                        if let Some(struct_type) = struct_type_opt {
                            // 使用 struct 类型进行 GEP 获取字段地址
                            let field_ptr = unsafe {
                                self.builder().build_in_bounds_gep(
                                    struct_type,
                                    ptr,
                                    &[zero, idx],
                                    member,
                                )?
                            };

                            // 加载字段值
                            let field_value =
                                self.builder().build_load(struct_type, field_ptr, member)?;
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
            Type::Struct(_) => self.context.ptr_type(AddressSpace::default()).into(),
            Type::Object(_) => self.context.ptr_type(AddressSpace::default()).into(),
        }
    }

    /// 生成 match 表达式
    /// 使用递归生成嵌套的 if-else 链
    fn generate_match(
        &mut self,
        value: &Expression,
        arms: &[MatchArm],
    ) -> Result<(), CodegenError> {
        let function = self.current_function.unwrap();
        let i64_type = self.context.i64_type();

        // 生成要匹配的值
        let match_value = self.generate_expression(value)?;

        // 获取匹配的值并统一转换为 i64 进行比较
        let match_int = if match_value.is_int_value() {
            // i32 扩展为 i64
            self.builder().build_int_s_extend(
                match_value.into_int_value(),
                i64_type,
                "match_i64",
            )?
        } else if match_value.is_pointer_value() {
            // 指针转换为 i64
            self.builder().build_ptr_to_int(
                match_value.into_pointer_value(),
                i64_type,
                "ptr_to_int",
            )?
        } else {
            return Err(CodegenError {
                message: "Match value must be integer or pointer type".to_string(),
            });
        };

        // 创建合并块
        let merge_block = self.context.append_basic_block(function, "match_merge");

        // 递归生成 if-else 链
        self.generate_match_arms(0, arms, match_int, i64_type, merge_block)?;

        // 设置合并块位置
        self.builder().position_at_end(merge_block);

        Ok(())
    }

    /// 递归生成 match 分支的 if-else 链
    fn generate_match_arms(
        &mut self,
        idx: usize,
        arms: &[MatchArm],
        match_int: inkwell::values::IntValue<'ctx>,
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
        let then_block = self.context.append_basic_block(function, "match_then");

        // 确定 else 目标
        let else_block = if idx + 1 < arms.len() {
            // 还有更多分支，需要继续检查
            self.context.append_basic_block(function, "match_else")
        } else {
            // 最后一个分支，跳转到 merge
            merge_block
        };

        // 生成条件比较
        let cond_i1: inkwell::values::IntValue<'ctx> = match &arm.pattern {
            MatchPattern::Number(n) => {
                let n_int = i64_type.const_int(*n as u64, false);
                self.builder().build_int_compare(
                    inkwell::IntPredicate::EQ,
                    match_int,
                    n_int,
                    "match_cond",
                )?
            },
            MatchPattern::Identifier(name) => {
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
                            message: "Match identifier must be integer or pointer type".to_string(),
                        });
                    };
                    self.builder().build_int_compare(
                        inkwell::IntPredicate::EQ,
                        match_int,
                        loaded_int,
                        "match_cond",
                    )?
                } else {
                    // 变量不存在，条件为 false
                    i64_type.const_int(0, false)
                }
            },
            MatchPattern::Wildcard => {
                // 通配符：条件始终为 true
                // 创建全 1 的 i64 值然后转换为 i1
                let one = i64_type.const_int(1, false);
                let zero = i64_type.const_int(0, false);
                self.builder().build_int_compare(
                    inkwell::IntPredicate::NE,
                    one,
                    zero,
                    "wildcard_cond",
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
            self.generate_match_arms(idx + 1, arms, match_int, i64_type, merge_block)?;
        }

        Ok(())
    }
}
