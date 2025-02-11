use super::CodeGenerator;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;
use oxc_ast::ast::Type;

impl<'ctx> CodeGenerator<'ctx> {
    pub(crate) fn get_llvm_type(&self, ts_type: &Type) -> BasicTypeEnum<'ctx> {
        match ts_type {
            Type::Number => self.context.i64_type().into(),
            Type::Boolean => self.context.bool_type().into(),
            Type::Void => self.context.void_type().into(),
            _ => self.context.i64_type().into(), // 默认类型
        }
    }

    fn convert_type(
        &self,
        from: BasicValueEnum<'ctx>,
        to: BasicTypeEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match (from.get_type(), to) {
            (f, t) if f == t => Ok(from),
            (f, t) => {
                if f.is_int_type() && t.is_int_type() {
                    Ok(self
                        .builder
                        .build_int_cast(from.into_int_value(), t.into_int_type(), "int_cast")
                        .map_err(|e| e.to_string())?
                        .into())
                } else {
                    Err("Unsupported type conversion".to_string())
                }
            }
        }
    }
}
