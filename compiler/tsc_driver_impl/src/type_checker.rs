use crate::symbol_table::SymbolTable;
use crate::types::Type;
use oxc_ast::ast::*;

pub struct TypeChecker {
    symbol_table: SymbolTable,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            symbol_table: SymbolTable::new(),
        }
    }

    pub fn check(&mut self, program: &Program) -> Result<Type, String> {
        for stmt in &program.body {
            match stmt {
                Statement::VariableDeclaration(var_decl) => {
                    for decl in &var_decl.declarations {
                        if let Some(init) = &decl.init {
                            self.check_expression(init)?;
                        }
                    }
                }
                Statement::ExpressionStatement(expr_stmt) => {
                    self.check_expression(&expr_stmt.expression)?;
                }
                _ => return Err("Unsupported statement type".to_string()),
            }
        }
        Ok(Type::Void)
    }

    fn check_expression(&mut self, expr: &Expression) -> Result<Type, String> {
        match expr {
            Expression::Identifier(ident) => Ok(Type::Unknown),
            Expression::NumericLiteral(num) => Ok(Type::Number),
            Expression::CallExpression(call) => {
                if let Expression::StaticMemberExpression(member) = &call.callee {
                    if let Expression::Identifier(obj) = &member.object {
                        if obj.name == "console" {
                            return Ok(Type::Void);
                        }
                    }
                }
                Err("Unsupported function call".to_string())
            }
            _ => Err("Unsupported expression type".to_string()),
        }
    }
}
