//! 模块系统
//!
//! 提供模块加载、符号解析和多文件编译支持。

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::ast::*;
use crate::lexer::Lexer;
use crate::parser::Parser;

/// 模块加载错误
#[derive(Debug)]
pub enum ModuleError {
    IoError(String),
    ParseError(String),
    ModuleNotFound(String),
    SymbolNotFound(String),
    CircularDependency(String),
}

/// 模块信息
#[derive(Debug, Clone)]
pub struct Module {
    /// 模块路径 (如 "std/io")
    pub path: String,
    /// 模块文件路径
    pub file_path: String,
    /// 解析后的 AST
    pub program: Program,
    /// 导出的符号表
    pub exports: HashMap<String, ExportedSymbol>,
}

/// 导出的符号
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ExportedSymbol {
    Function(Box<Function>),
    Variable(String, Type),
    Struct(StructDefinition),
    Constant(String),
}

/// 符号表
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    /// 符号映射 (名称 -> 符号)
    symbols: HashMap<String, ExportedSymbol>,
    /// 别名映射
    aliases: HashMap<String, String>,
}

impl SymbolTable {
    /// 创建新的符号表
    pub fn new() -> Self {
        Self { symbols: HashMap::new(), aliases: HashMap::new() }
    }

    /// 添加符号
    pub fn add(&mut self, name: String, symbol: ExportedSymbol) {
        self.symbols.insert(name, symbol);
    }

    /// 添加别名
    pub fn add_alias(&mut self, original: String, alias: String) {
        self.aliases.insert(alias, original);
    }

    /// 查找符号 (支持别名)
    pub fn find(&self, name: &str) -> Option<ExportedSymbol> {
        // 先查找别名
        if let Some(original) = self.aliases.get(name) {
            return self.symbols.get(original).cloned();
        }
        // 直接查找
        self.symbols.get(name).cloned()
    }

    /// 获取所有符号名称
    pub fn symbols(&self) -> Vec<String> {
        self.symbols.keys().cloned().collect()
    }
}

/// 模块加载器
pub struct ModuleLoader {
    /// 已加载的模块缓存
    loaded_modules: HashMap<String, Module>,
    /// 模块搜索路径
    search_paths: Vec<String>,
    /// 全局符号表
    global_symbols: HashMap<String, SymbolTable>,
}

impl ModuleLoader {
    /// 创建新的模块加载器
    pub fn new() -> Self {
        let search_paths = vec![
            ".".to_string(),
            "examples".to_string(),
            "examples/std".to_string(),
            "compiler/nexac/std".to_string(),
        ];

        Self { loaded_modules: HashMap::new(), search_paths, global_symbols: HashMap::new() }
    }

    /// 添加搜索路径
    pub fn add_search_path(&mut self, path: String) {
        self.search_paths.push(path);
    }

    /// 加载模块
    pub fn load_module(&mut self, module_path: &str) -> Result<Module, ModuleError> {
        // 检查缓存
        if let Some(module) = self.loaded_modules.get(module_path) {
            return Ok(module.clone());
        }

        // 查找模块文件
        let file_path = self.find_module_file(module_path)?;

        // 读取源文件
        let source =
            fs::read_to_string(&file_path).map_err(|e| ModuleError::IoError(e.to_string()))?;

        // 解析模块
        let _lexer = Lexer::new(&source);
        let mut parser = Parser::new(&source);
        let program = parser.parse_program().map_err(|e| ModuleError::ParseError(e.message))?;

        // 提取导出的符号
        let exports = self.extract_exports(&program);

        // 创建模块
        let module = Module { path: module_path.to_string(), file_path, program, exports };

        // 缓存模块
        let result = module.clone();
        self.loaded_modules.insert(module_path.to_string(), module);

        Ok(result)
    }

    /// 查找模块文件
    fn find_module_file(&self, module_path: &str) -> Result<String, ModuleError> {
        // 尝试不同的文件扩展名
        let extensions = ["ts", "nexa", "nex"];

        // 将模块路径转换为文件路径
        let relative_path = module_path.replace('.', "/");

        for search_path in &self.search_paths {
            for ext in &extensions {
                let file_path = format!("{}/{}.{}", search_path, relative_path, ext);
                if Path::new(&file_path).exists() {
                    return Ok(file_path);
                }
            }
        }

        Err(ModuleError::ModuleNotFound(format!(
            "Module '{}' not found in search paths: {:?}",
            module_path, self.search_paths
        )))
    }

    /// 从 Program 中提取导出的符号
    fn extract_exports(&self, program: &Program) -> HashMap<String, ExportedSymbol> {
        let mut exports = HashMap::new();

        // 处理函数导出
        for func in &program.functions {
            // 检查函数是否在导出列表中
            let is_exported = program.exports.iter().any(|exp| match &exp.kind {
                ExportKind::Named(specs) => specs.iter().any(|s| s.name == func.name),
                _ => false,
            });

            // 如果有 export 声明且函数被列出，则导出
            // 否则默认导出所有函数（暂时这样处理）
            if !program.exports.is_empty() && is_exported {
                exports.insert(func.name.clone(), ExportedSymbol::Function(Box::new(func.clone())));
            } else if program.exports.is_empty() {
                // 没有显式导出，默认导出所有函数
                exports.insert(func.name.clone(), ExportedSymbol::Function(Box::new(func.clone())));
            }
        }

        // 处理变量导出
        for stmt in &program.statements {
            if let Statement::VariableDeclaration {
                name,
                type_annotation,
                initializer,
                mutable: _,
                span: _,
            } = stmt
            {
                // 检查是否在导出列表中
                let is_exported = program.exports.iter().any(|exp| match &exp.kind {
                    ExportKind::Named(specs) => specs.iter().any(|s| s.name == *name),
                    _ => false,
                });

                if !program.exports.is_empty() && is_exported {
                    let var_type = type_annotation.clone().unwrap_or(Type::Number);
                    exports.insert(name.clone(), ExportedSymbol::Variable(name.clone(), var_type));
                } else if program.exports.is_empty() && initializer.is_some() {
                    // 默认导出带初始化的变量
                    let var_type = type_annotation.clone().unwrap_or(Type::Number);
                    exports.insert(name.clone(), ExportedSymbol::Variable(name.clone(), var_type));
                }
            }
        }

        // 处理 struct 导出
        for struct_def in &program.structs {
            exports.insert(struct_def.name.clone(), ExportedSymbol::Struct(struct_def.clone()));
        }

        exports
    }

    /// 解析导入并返回符号表
    pub fn resolve_imports(
        &mut self,
        program: &Program,
    ) -> Result<HashMap<String, SymbolTable>, ModuleError> {
        let mut import_symbols = HashMap::new();

        for import in &program.imports {
            // 加载导入的模块
            let module = self.load_module(&import.module_path)?;

            // 创建模块的符号表
            let mut symbols = SymbolTable::new();

            // 处理命名空间导入 (import * as ns)
            if let Some(alias) = &import.alias {
                symbols = module.exports.clone().into();
                self.global_symbols.insert(alias.clone(), symbols.clone());
            }
            // 处理命名导入 (import { foo, bar })
            else if !import.imports.is_empty() {
                for spec in &import.imports {
                    let name = &spec.name;
                    if let Some(symbol) = module.exports.get(name) {
                        let target_name = spec.alias.as_ref().unwrap_or(name);
                        symbols.add(target_name.clone(), symbol.clone());
                    }
                }
                // 使用模块路径作为命名空间键
                import_symbols.insert(import.module_path.clone(), symbols);
            }
            // 处理默认导入 (import foo from "module")
            else if import.alias.is_some() {
                let default_name = import.alias.as_ref().unwrap();
                // 默认导入第一个导出的符号
                if let Some((_name, symbol)) = module.exports.iter().next() {
                    symbols.add(default_name.clone(), symbol.clone());
                }
                import_symbols.insert(import.module_path.clone(), symbols);
            }
        }

        Ok(import_symbols)
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, ExportedSymbol>> for SymbolTable {
    fn from(map: HashMap<String, ExportedSymbol>) -> Self {
        Self { symbols: map, aliases: HashMap::new() }
    }
}
