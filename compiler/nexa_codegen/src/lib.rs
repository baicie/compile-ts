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
    /// 类型映射器
    #[allow(dead_code)]
    type_mapper: TypeMapper<'ctx>,
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
            type_mapper,
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
            match func.return_type {
                Type::Void => {
                    self.builder().build_return(None)?;
                },
                _ => {
                    // 返回默认值
                    let default = self.get_default_value(&func.return_type);
                    self.builder().build_return(Some(&default))?;
                },
            }
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
                let i32_type = self.context.i32_type();
                let zero = i32_type.const_int(0, false);
                let condition_i1 = self.builder().build_int_compare(
                    inkwell::IntPredicate::NE,
                    condition_value.into_int_value(),
                    zero,
                    "condition_cmp",
                )?;

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
                let i32_type = self.context.i32_type();
                let zero = i32_type.const_int(0, false);
                let condition_i1 = self.builder().build_int_compare(
                    inkwell::IntPredicate::NE,
                    condition_value.into_int_value(),
                    zero,
                    "condition_cmp",
                )?;
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
                    let i32_type = self.context.i32_type();
                    let zero = i32_type.const_int(0, false);
                    let condition_i1 = self.builder().build_int_compare(
                        inkwell::IntPredicate::NE,
                        condition_value.into_int_value(),
                        zero,
                        "condition_cmp",
                    )?;
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
                    let ptr_type: BasicTypeEnum<'ctx> = ptr.get_type().into();
                    let value = self.builder().build_load(ptr_type, ptr, name)?;
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
            Expression::Member { object: _, member, span: _ } => {
                Err(CodegenError { message: format!("Member access not implemented: {}", member) })
            },
            Expression::Assignment { target, value, span: _ } => {
                let value = self.generate_expression(value)?;
                if let Expression::Identifier(name, _) = target.as_ref() {
                    if let Some(ptr) = self.get_variable(name) {
                        let basic_value = value;
                        self.builder().build_store(ptr, basic_value)?;
                    }
                }
                Ok(value)
            },
        }
    }

    /// 将 Nexa 类型映射到 LLVM 类型
    fn map_type(&self, ty: &Type) -> BasicTypeEnum<'ctx> {
        match ty {
            Type::I32 => self.context.i32_type().into(),
            Type::I64 => self.context.i64_type().into(),
            Type::F32 => self.context.f32_type().into(),
            Type::F64 => self.context.f64_type().into(),
            Type::Bool => self.context.i32_type().into(),
            Type::String => self.context.ptr_type(AddressSpace::default()).into(),
            Type::Void => {
                // Void 类型作为函数返回类型时不映射到 BasicTypeEnum
                // 这里返回 i32 作为占位符
                self.context.i32_type().into()
            },
            Type::Array(_) => self.context.ptr_type(AddressSpace::default()).into(),
            Type::Pointer(_) => self.context.ptr_type(AddressSpace::default()).into(),
            Type::Function(_, return_type) => self.map_type(return_type),
        }
    }

    /// 获取类型的默认值
    fn get_default_value(&self, ty: &Type) -> BasicValueEnum<'ctx> {
        match ty {
            Type::I32 => self.context.i32_type().const_int(0, false).into(),
            Type::I64 => self.context.i64_type().const_int(0, false).into(),
            Type::F32 => self.context.f32_type().const_float(0.0).into(),
            Type::F64 => self.context.f64_type().const_float(0.0).into(),
            Type::Bool => self.context.i32_type().const_int(0, false).into(),
            Type::String => self.context.ptr_type(AddressSpace::default()).const_null().into(),
            Type::Void => self.context.i32_type().const_int(0, false).into(),
            _ => self.context.i32_type().const_int(0, false).into(),
        }
    }
}
