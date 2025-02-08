use crate::types::Type;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Symbol {
    name: String,
    typ: Type,
    is_mutable: bool,
}

pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String, typ: Type, is_mutable: bool) {
        let symbol = Symbol {
            name: name.clone(),
            typ,
            is_mutable,
        };
        self.scopes.last_mut().unwrap().insert(name, symbol);
    }
}
