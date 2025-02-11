use oxc_ast::ast::{ExpressionStatement, Statement, VariableDeclaration};

impl<'ctx> super::CodeGenerator<'ctx> {
    pub(crate) fn generate_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                self.generate_variable_declaration(var_decl)
            }
            Statement::ExpressionStatement(expr_stmt) => {
                self.generate_expression_statement(expr_stmt)
            }
            _ => Err("Unsupported statement".to_string()),
        }
    }

    fn generate_variable_declaration(
        &mut self,
        var_decl: &VariableDeclaration,
    ) -> Result<(), String> {
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

    fn generate_expression_statement(
        &mut self,
        expr_stmt: &ExpressionStatement,
    ) -> Result<(), String> {
        self.generate_expression(&expr_stmt.expression)?;
        Ok(())
    }
}
