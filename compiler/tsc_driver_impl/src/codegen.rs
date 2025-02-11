use crate::gc::GarbageCollector;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{InitializationConfig, Target, TargetMachine, TargetTriple};
use inkwell::values::BasicValueEnum;
use inkwell::OptimizationLevel;
use oxc_ast::ast::{Expression, Program, Statement};
use std::collections::HashMap;

pub struct CodeGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, inkwell::values::PointerValue<'ctx>>,
    gc: GarbageCollector<'ctx>,
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
            gc: GarbageCollector::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<(), String> {
        // 创建主函数
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        // 生成语句
        for stmt in &program.body {
            self.generate_statement(stmt)?;
        }

        // 添加返回语句
        self.builder
            .build_return(Some(&i64_type.const_int(0, false)))
            .map_err(|e| e.to_string())?;

        Ok(())
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

    fn generate_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                for decl in &var_decl.declarations {
                    if let Some(init) = &decl.init {
                        let value = self.generate_expression(init)?;
                        let name = decl.id.get_identifier_name().unwrap().to_string();
                        let alloca = self
                            .builder
                            .build_alloca(self.context.i64_type(), &name)
                            .map_err(|e| e.to_string())?;

                        // 将对象添加到 GC
                        self.gc.alloc_object(&name, alloca);

                        self.builder
                            .build_store(alloca, value)
                            .map_err(|e| e.to_string())?;
                        self.variables.insert(name.clone(), alloca);
                    }
                }
                Ok(())
            }
            Statement::ExpressionStatement(expr_stmt) => {
                self.generate_expression(&expr_stmt.expression)?;
                Ok(())
            }
            _ => Err(format!("Unsupported statement type: {:?}", stmt)),
        }
    }

    fn generate_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expression::NumericLiteral(num) => Ok(self
                .context
                .i64_type()
                .const_int(num.value as u64, false)
                .into()),
            Expression::Identifier(ident) => {
                if let Some(val) = self.variables.get(ident.name.as_str()) {
                    Ok(self
                        .builder
                        .build_load(self.context.i64_type(), *val, ident.name.as_str())
                        .map_err(|e| e.to_string())?)
                } else {
                    Err(format!("Undefined variable: {}", ident.name))
                }
            }
            Expression::CallExpression(call) => {
                if let Expression::StaticMemberExpression(member) = &call.callee {
                    if let Expression::Identifier(obj) = &member.object {
                        if obj.name.as_str() == "console" && member.property.name.as_str() == "log"
                        {
                            if let Some(arg) = call.arguments.first() {
                                if let Some(expr) = arg.as_expression() {
                                    let value = self.generate_expression(expr)?;
                                    self.generate_console_log(value)?;
                                    return Ok(self.context.i64_type().const_int(0, false).into());
                                }
                            }
                        }
                    }
                }
                Err("Unsupported function call".to_string())
            }
            _ => Err("Unsupported expression type".to_string()),
        }
    }

    pub fn print_to_file(&self, filename: &str) {
        self.module
            .print_to_file(filename)
            .expect("Failed to write IR to file");
    }

    // pub fn optimize(&self) {
    //     use inkwell::passes::PassManager;
    //     let pass_manager = PassManager::create(&self.module);
    //     pass_manager.initialize();
    //     pass_manager.run_on_module(&self.module);
    // }

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

impl<'ctx> Drop for CodeGenerator<'ctx> {
    fn drop(&mut self) {
        // 清理所有堆对象
        for name in self.variables.keys() {
            if self.gc.decrement_ref(name) {
                // 对象的引用计数为0，可以释放
                println!("Cleaning up object: {}", name);
            }
        }
    }
}
