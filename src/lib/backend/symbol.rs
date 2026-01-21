use super::mem::Location;
use crate::{backend::mem::Register, game::SignalId};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Default)]
pub struct Scope {
    pub table: RefCell<SymbolTable>,
}

pub type SharedScope = Rc<RefCell<Scope>>;

#[derive(Debug, Default)]
pub struct ScopeStack {
    scopes: Vec<SharedScope>,
}

impl ScopeStack {
    pub fn lookup_name(&self, name: &String) -> Option<SymbolHandle> {
        for scope in self.scopes.iter().rev() {
            let scope = scope.borrow();
            let table = scope.table.borrow();
            if let Some(sym_ref) = table.lookup_name(name)
                && &sym_ref.sym.borrow().name == name
            {
                return Some(sym_ref);
            }
        }

        None
    }

    pub fn enter_scope(&mut self) -> SharedScope {
        self.scopes.push(Rc::new(RefCell::new(Scope::default())));
        self.scopes.last().unwrap().clone()
    }

    pub fn leave_scope(&mut self) -> Option<SharedScope> {
        self.scopes.pop()
    }
}

#[derive(Debug, Clone)]
pub struct SymbolHandle {
    pub sid: SymbolId,
    pub sym: SharedSymbol,
}

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub loc: Location,
    // pub depth: usize,
    pub signal: Option<SignalId>,
}

impl Symbol {
    pub fn new(name: String, loc: Location, signal: Option<SignalId>) -> Self {
        Self {
            name,
            loc,
            signal,
            // depth,
        }
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

type SharedSymbol = Rc<RefCell<Symbol>>;

#[derive(Debug)]
pub struct SymbolTable {
    storage: HashMap<SymbolId, SharedSymbol>,
    field: HashMap<String, SymbolId>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            field: HashMap::new(),
        }
    }

    pub fn insert(&mut self, sid: SymbolId, sym: Symbol) {
        let indent = sym.name.clone();
        self.storage.insert(sid, Rc::new(RefCell::new(sym)));
        self.field.insert(indent, sid);
    }

    pub fn lookup_register(&self, reg: &Register) -> Option<SharedSymbol> {
        self.storage
            .iter()
            .find(|(_, sym)| {
                let sym = sym.borrow();
                match sym.loc {
                    Location::Reg(r) => &r == reg,
                    _ => false,
                }
            })
            .map(|(_, sym)| Rc::clone(&sym))
    }

    pub fn lookup_name(&self, name: &String) -> Option<SymbolHandle> {
        let sid = self.field.get(name)?;
        self.storage.get(sid).map(|sym| SymbolHandle {
            sid: *sid,
            sym: sym.clone(),
        })
    }
}
