use super::CodeGenerator;
use inkwell::values::FunctionValue;
use oxc_ast::ast::{Function, Parameter, Statement};

impl<'ctx> CodeGenerator<'ctx> {
    pub(crate) fn generate_function(&mut self, func: &Function) -> Result<(), String> {
        let fn_type = self.context.i64_type().fn_type(&[], false);
        let function = self.module.add_function(&func.id.name, fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);
        self.generate_function_parameters(function, &func.params)?;
        self.generate_function_body(&func.body)?;

        Ok(())
    }

    fn generate_function_body(&mut self, body: &[Statement]) -> Result<(), String> {
        for stmt in body {
            self.generate_statement(stmt)?;
        }
        Ok(())
    }

    fn generate_function_parameters(
        &mut self,
        function: FunctionValue<'ctx>,
        params: &[Parameter],
    ) -> Result<(), String> {
        for (i, param) in params.iter().enumerate() {
            let param_name = param.name.as_str();
            let alloca = self
                .builder
                .build_alloca(self.context.i64_type(), param_name)
                .map_err(|e| e.to_string())?;

            self.builder
                .build_store(alloca, function.get_nth_param(i as u32).unwrap())
                .map_err(|e| e.to_string())?;

            self.variables.insert(param_name.to_string(), alloca);
        }
        Ok(())
    }
}
