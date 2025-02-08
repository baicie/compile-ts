use crate::parser::AstNode;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{InitializationConfig, Target, TargetMachine, TargetTriple};
use inkwell::values::BasicValueEnum;
use inkwell::OptimizationLevel;
use std::collections::HashMap;

pub struct CodeGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, inkwell::values::PointerValue<'ctx>>,
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        // 声明 printf 函数
        let printf_type = context.i32_type().fn_type(
            &[context
                .i8_type()
                .ptr_type(inkwell::AddressSpace::default())
                .into()],
            true,
        );
        module.add_function("printf", printf_type, None);

        CodeGenerator {
            context,
            module,
            builder,
            variables: HashMap::new(),
        }
    }

    pub fn generate(&mut self, ast: &AstNode) -> Result<(), String> {
        match ast {
            AstNode::Program(statements) => {
                // 创建主函数
                let i64_type = self.context.i64_type();
                let fn_type = i64_type.fn_type(&[], false);
                let function = self.module.add_function("main", fn_type, None);
                let basic_block = self.context.append_basic_block(function, "entry");

                // 设置插入点
                self.builder.position_at_end(basic_block);

                // 生成语句
                for stmt in statements {
                    self.generate_statement(stmt)?;
                }

                // 添加返回语句
                self.builder
                    .build_return(Some(&i64_type.const_int(0, false)))
                    .map_err(|e| e.to_string())?;

                Ok(())
            }
            _ => Err("Expected program".to_string()),
        }
    }

    fn generate_console_log(&self, value: BasicValueEnum<'ctx>) -> Result<(), String> {
        // 创建格式字符串 "%d\n"
        let format_str = self
            .builder
            .build_global_string_ptr("%d\n", "format_str")
            .map_err(|e| e.to_string())?;

        // 获取 printf 函数
        let printf = self.module.get_function("printf").unwrap();

        // 调用 printf
        self.builder
            .build_call(
                printf,
                &[format_str.as_pointer_value().into(), value.into()],
                "printf_call",
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn generate_statement(&mut self, node: &AstNode) -> Result<(), String> {
        match node {
            AstNode::FunctionDecl { name, params, body } => {
                let i64_type = self.context.i64_type();
                let param_types: Vec<_> = params.iter().map(|_| i64_type.into()).collect();
                let fn_type = i64_type.fn_type(&param_types, false);

                let function = self.module.add_function(name, fn_type, None);
                let basic_block = self.context.append_basic_block(function, "entry");
                self.builder.position_at_end(basic_block);

                for stmt in body {
                    self.generate_statement(stmt)?;
                }
                Ok(())
            }
            AstNode::ReturnStmt(expr) => {
                let return_value = self.generate_expression(expr)?;
                self.builder
                    .build_return(Some(&return_value))
                    .map_err(|e| e.to_string())?;
                Ok(())
            }
            AstNode::VariableDecl { name, initializer } => {
                let value = self.generate_expression(initializer)?;
                let alloca = self
                    .builder
                    .build_alloca(self.context.i64_type(), name)
                    .map_err(|e| e.to_string())?;
                self.builder
                    .build_store(alloca, value)
                    .map_err(|e| e.to_string())?;
                self.variables.insert(name.clone(), alloca);
                Ok(())
            }
            AstNode::ExpressionStmt(expr) => {
                self.generate_expression(expr)?;
                Ok(())
            }
            AstNode::CallExpr { callee, args } => {
                if callee == "console.log" {
                    if let Some(arg) = args.first() {
                        let value = self.generate_expression(arg)?;
                        self.generate_console_log(value)?;
                    }
                    Ok(())
                } else {
                    Err("Unsupported function call".to_string())
                }
            }
            _ => Err(format!("Unsupported statement type: {:?}", node)),
        }
    }

    fn generate_expression(&mut self, node: &AstNode) -> Result<BasicValueEnum<'ctx>, String> {
        match node {
            AstNode::BinaryExpr {
                left,
                operator,
                right,
            } => {
                let l = self.generate_expression(left)?;
                let r = self.generate_expression(right)?;

                match operator.as_str() {
                    "Plus" => Ok(self
                        .builder
                        .build_int_add(l.into_int_value(), r.into_int_value(), "addtmp")
                        .map_err(|e| e.to_string())?
                        .into()),
                    "Minus" => Ok(self
                        .builder
                        .build_int_sub(l.into_int_value(), r.into_int_value(), "subtmp")
                        .map_err(|e| e.to_string())?
                        .into()),
                    _ => Err("Unsupported operator".to_string()),
                }
            }
            AstNode::NumberLiteral(n) => {
                Ok(self.context.i64_type().const_int(*n as u64, false).into())
            }
            AstNode::Identifier(name) => {
                if let Some(val) = self.variables.get(name) {
                    Ok(self
                        .builder
                        .build_load(*val, name)
                        .map_err(|e| e.to_string())?)
                } else {
                    Err(format!("Undefined variable: {}", name))
                }
            }
            AstNode::CallExpr { callee, args } => {
                if callee == "console.log" {
                    if let Some(arg) = args.first() {
                        let value = self.generate_expression(arg)?;
                        self.generate_console_log(value)?;
                        Ok(self.context.i64_type().const_int(0, false).into())
                    } else {
                        Err("console.log requires an argument".to_string())
                    }
                } else {
                    Err("Unsupported function call".to_string())
                }
            }
            _ => Err("Unsupported expression type".to_string()),
        }
    }

    pub fn print_to_file(&self, filename: &str) {
        self.module
            .print_to_file(filename)
            .expect("Failed to write IR to file");
    }

    pub fn optimize(&self) {
        use inkwell::passes::PassManagerBuilder;

        let builder = PassManagerBuilder::create();
        builder.set_optimization_level(OptimizationLevel::Aggressive);

        let pass_manager = inkwell::passes::PassManager::create(());
        builder.populate_module_pass_manager(&pass_manager);
        pass_manager.run_on(&self.module);
    }

    pub fn generate_object_file(
        &self,
        filename: &str,
        target_triple: Option<String>,
    ) -> Result<(), String> {
        // 初始化所有目标平台
        Target::initialize_all(&InitializationConfig::default());

        // 获取目标平台
        let target_triple = target_triple
            .map(|t| TargetTriple::create(&t))
            .unwrap_or_else(TargetMachine::get_default_triple);

        let target = Target::from_triple(&target_triple).map_err(|e| e.to_string())?;

        // 创建目标机器
        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                OptimizationLevel::Default,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .ok_or("Could not create target machine")?;

        // 生成目标文件
        target_machine
            .write_to_file(
                &self.module,
                inkwell::targets::FileType::Object,
                filename.as_ref(),
            )
            .map_err(|e| e.to_string())
    }
}
