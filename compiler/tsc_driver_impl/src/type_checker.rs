use crate::parser::AstNode;
use crate::symbol_table::SymbolTable;
use crate::types::Type;

pub struct TypeChecker {
    symbol_table: SymbolTable,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            symbol_table: SymbolTable::new(),
        }
    }

    pub fn check(&mut self, ast: &AstNode) -> Result<Type, String> {
        match ast {
            AstNode::Program(statements) => {
                for stmt in statements {
                    self.check(stmt)?;
                }
                Ok(Type::Void)
            }
            AstNode::VarDecl { init, typ, .. } => {
                if let Some(init) = init {
                    self.check(init)?;
                }
                Ok(typ
                    .as_ref()
                    .map(|t| Type::from(t.as_str()))
                    .unwrap_or(Type::Unknown))
            }
            AstNode::FunctionDecl { name, params, body } => {
                self.symbol_table.enter_scope();
                let return_type = self.check_function_body(body)?;
                self.symbol_table.exit_scope();
                Ok(return_type)
            }
            AstNode::VariableDecl { name, initializer } => {
                let init_type = self.check(initializer)?;
                self.symbol_table
                    .define(name.clone(), init_type.clone(), true);
                Ok(init_type)
            }
            AstNode::ReturnStmt(expr) => self.check(expr),
            AstNode::ExpressionStmt(expr) => self.check(expr),
            AstNode::BinaryExpr {
                left,
                operator: _,
                right,
            } => {
                let left_type = self.check(left)?;
                let right_type = self.check(right)?;
                if left_type == right_type {
                    Ok(left_type)
                } else {
                    Err("Type mismatch in binary expression".to_string())
                }
            }
            AstNode::Identifier(name) => Ok(Type::Unknown),
            AstNode::NumberLiteral(_) => Ok(Type::Number),
            AstNode::CallExpr { callee, args } => {
                if callee == "console.log" {
                    if let Some(arg) = args.first() {
                        self.check(arg)?;
                    }
                    Ok(Type::Void)
                } else {
                    Err(format!("Unknown function: {}", callee))
                }
            }
        }
    }

    fn check_function_body(&mut self, body: &[AstNode]) -> Result<Type, String> {
        for stmt in body {
            self.check(stmt)?;
        }
        // 简化版：假设最后一个语句是返回语句
        if let Some(last_stmt) = body.last() {
            self.check(last_stmt)
        } else {
            Ok(Type::Void)
        }
    }
}
