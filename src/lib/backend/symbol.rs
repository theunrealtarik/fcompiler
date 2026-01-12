use std::collections::HashMap;

type VariableMap = HashMap<String, super::mem::Variable>;

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

    pub fn get_by_register(
        &self,
        reg: &crate::backend::mem::Register,
    ) -> Option<&crate::backend::mem::Variable> {
        self.iter()
            .find(|(_, var)| match var.loc {
                crate::backend::mem::Location::REG(r) => r == *reg,
                _ => false,
            })
            .map(|(_, v)| v)
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
