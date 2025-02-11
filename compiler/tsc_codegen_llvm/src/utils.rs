use super::CodeGenerator;
use inkwell::values::{BasicValueEnum, PointerValue};

impl<'ctx> CodeGenerator<'ctx> {
    pub(crate) fn create_entry_block_alloca(&self, name: &str) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap()
            .get_first_basic_block()
            .unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder
            .build_alloca(self.context.i64_type(), name)
            .expect("Failed to create entry block allocation")
    }

    pub(crate) fn generate_console_log(&self, value: BasicValueEnum<'ctx>) -> Result<(), String> {
        let format_str = self
            .builder
            .build_global_string_ptr("%d\n", "format_str")
            .map_err(|e| e.to_string())?;

        let printf = self.module.get_function("printf").unwrap();

        self.builder
            .build_call(
                printf,
                &[format_str.as_pointer_value().into(), value.into()],
                "printf_call",
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
