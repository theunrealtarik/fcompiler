use std::collections::HashMap;

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub loc: super::mem::Location,
    pub signal: Option<crate::game::SignalId>,
}

impl Symbol {
    pub fn new(
        name: String,
        loc: super::mem::Location,
        signal: Option<crate::game::SignalId>,
    ) -> Self {
        Self { name, loc, signal }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SymbolId(pub i32);

impl std::ops::Deref for SymbolId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    storage: HashMap<SymbolId, Symbol>,
    field: HashMap<String, SymbolId>,
}

impl std::ops::Deref for SymbolTable {
    type Target = HashMap<SymbolId, Symbol>;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl std::ops::DerefMut for SymbolTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            field: HashMap::new(),
        }
    }

    pub fn get_by_register(&self, reg: &crate::backend::mem::Register) -> Option<&Symbol> {
        self.iter()
            .find(|(_, var)| match var.loc {
                crate::backend::mem::Location::REG(r) => r == *reg,
                _ => false,
            })
            .map(|(_, v)| v)
    }

    pub fn push(&mut self, sid: &SymbolId, symbol: Symbol) {
        let indent = symbol.name.clone();
        self.storage.insert(*sid, symbol);
        self.field.insert(indent, *sid);
    }

    pub fn lookup(&self, name: &String) -> Option<(&SymbolId, &Symbol)> {
        let sid = self.field.get(name)?;
        self.storage.get(sid).map(|sym| (sid, sym))
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
