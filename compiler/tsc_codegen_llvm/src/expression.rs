use inkwell::values::BasicValueEnum;
use oxc_ast::ast::{
    CallExpression, Expression, Identifier, NumericLiteral, StaticMemberExpression,
};

impl<'ctx> super::CodeGenerator<'ctx> {
    pub(crate) fn generate_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expression::NumericLiteral(num) => self.generate_numeric_literal(num),
            Expression::Identifier(ident) => self.generate_identifier(ident),
            Expression::CallExpression(call) => self.generate_call_expression(call),
            _ => Err("Unsupported expression".to_string()),
        }
    }

    fn generate_numeric_literal(
        &self,
        num: &NumericLiteral,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        Ok(self
            .context
            .i64_type()
            .const_int(num.value as u64, false)
            .into())
    }

    fn generate_identifier(&self, ident: &Identifier) -> Result<BasicValueEnum<'ctx>, String> {
        if let Some(val) = self.variables.get(ident.name.as_str()) {
            Ok(self
                .builder
                .build_load(self.context.i64_type(), *val, ident.name.as_str())
                .map_err(|e| e.to_string())?)
        } else {
            Err(format!("Undefined variable: {}", ident.name))
        }
    }

    fn generate_call_expression(
        &mut self,
        call: &CallExpression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if let Expression::StaticMemberExpression(member) = &call.callee {
            if let Expression::Identifier(obj) = &member.object {
                if obj.name.as_str() == "console" && member.property.name.as_str() == "log" {
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
}
