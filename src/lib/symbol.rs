use std::collections::HashMap;

type VariableMap = HashMap<String, crate::mem::Variable>;

#[derive(Debug)]
pub struct SymbolTable(VariableMap);

impl std::ops::Deref for SymbolTable {
    type Target = VariableMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SymbolTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
