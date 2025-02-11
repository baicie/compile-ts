use crate::gc::GarbageCollector;
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    targets::{InitializationConfig, Target, TargetMachine, TargetTriple},
    values::PointerValue,
    OptimizationLevel,
};
use oxc_ast::ast::Program;
use std::collections::HashMap;

pub struct CodeGenerator<'ctx> {
    pub(crate) context: &'ctx Context,
    pub(crate) module: Module<'ctx>,
    pub(crate) builder: Builder<'ctx>,
    pub(crate) variables: HashMap<String, PointerValue<'ctx>>,
    pub(crate) gc: GarbageCollector<'ctx>,
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

    pub fn print_to_file(&self, filename: &str) {
        self.module
            .print_to_file(filename)
            .expect("Failed to write IR to file");
    }

    pub fn generate_object_file(
        &self,
        filename: &str,
        target_triple: Option<String>,
    ) -> Result<(), String> {
        Target::initialize_all(&InitializationConfig::default());

        let target_triple = target_triple
            .map(|t| TargetTriple::create(&t))
            .unwrap_or_else(TargetMachine::get_default_triple);

        let target = Target::from_triple(&target_triple).map_err(|e| e.to_string())?;

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

        target_machine
            .write_to_file(
                &self.module,
                inkwell::targets::FileType::Object,
                filename.as_ref(),
            )
            .map_err(|e| e.to_string())
    }
}
