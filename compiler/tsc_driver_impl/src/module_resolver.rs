use oxc_allocator::Allocator;
use oxc_ast::ast::{Program, Statement};
use oxc_parser::Parser;
use oxc_resolver::Resolver;
use oxc_span::SourceType;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ModuleResolver<'a> {
    resolver: Resolver,
    modules: HashMap<PathBuf, Program<'a>>,
    entry_point: PathBuf,
    allocator: &'a Allocator,
}

impl<'a> ModuleResolver<'a> {
    pub fn new(entry_point: PathBuf) -> Self {
        let allocator = Box::leak(Box::new(Allocator::default()));
        ModuleResolver {
            resolver: Resolver::default(),
            modules: HashMap::new(),
            entry_point,
            allocator,
        }
    }

    pub fn resolve_all(&mut self) -> Result<(), String> {
        let mut to_resolve = vec![self.entry_point.clone()];

        while let Some(path) = to_resolve.pop() {
            if self.modules.contains_key(&path) {
                continue;
            }

            let source = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

            let source_type = SourceType::from_path(&path).unwrap();
            let source = self.allocator.alloc_str(&source);
            let parser = Parser::new(&self.allocator, source, source_type);
            let ret = parser.parse();

            if !ret.errors.is_empty() {
                return Err(format!("Parse errors in {}", path.display()));
            }

            // 收集导入语句
            for stmt in &ret.program.body {
                if let Statement::ImportDeclaration(import) = stmt {
                    let source = &import.source.value;
                    if let Ok(resolved) = self.resolver.resolve(&path, source) {
                        to_resolve.push(resolved.into_path_buf());
                    }
                }
            }

            self.modules.insert(path, ret.program);
        }

        Ok(())
    }

    pub fn get_all_modules(&self) -> &HashMap<PathBuf, Program> {
        &self.modules
    }
}
