use super::mem::Location;
use crate::{
    backend::{asm::Label, mem::Register},
    game::SignalId,
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Default, Clone, Copy, strum_macros::EnumIs)]
pub enum ScopeKind {
    Global,
    #[default]
    Local,
    Then,
    Else,
    For,
    While,
    Loop,
}

#[derive(Debug, Default, derive_builder::Builder)]
pub struct ScopeMetadata {
    #[builder(setter(skip = true))]
    depth: usize,
    pub kind: ScopeKind,
    #[builder(setter(into, strip_option), default = None)]
    pub start_label: Option<Label>,
    #[builder(setter(into, strip_option), default = None)]
    pub exit_label: Option<Label>,
}

#[derive(Debug, Default)]
pub struct Scope {
    pub locals: RefCell<SymbolTable>,
    pub metadata: ScopeMetadata,
}

pub type SharedScope = Rc<RefCell<Scope>>;

#[derive(Debug, Default)]
pub struct ScopeStack {
    scopes: Vec<SharedScope>,
    birdeye: HashMap<SymbolId, SharedSymbol>,
}

impl ScopeStack {
    pub fn lookup_name(&self, name: &String) -> Option<SymbolHandle> {
        for scope in self.scopes.iter().rev() {
            let scope = scope.borrow();
            let table = scope.locals.borrow();
            if let Some(sym_ref) = table.lookup_name(name)
                && &sym_ref.sym.borrow().name == name
            {
                return Some(sym_ref);
            }
        }

        None
    }

    pub fn ladder(&self) -> std::iter::Rev<std::slice::Iter<'_, Rc<RefCell<Scope>>>> {
        self.scopes.iter().rev()
    }

    pub fn birdeye(&self) -> &HashMap<SymbolId, SharedSymbol> {
        &self.birdeye
    }

    pub fn enter_scope(&mut self) -> SharedScope {
        self.enter_scope_explicit(ScopeMetadata::default())
    }

    pub fn enter_scope_explicit(&mut self, metadata: ScopeMetadata) -> SharedScope {
        let scope = Scope {
            metadata: ScopeMetadata {
                kind: if self.scopes.is_empty() {
                    ScopeKind::Global
                } else {
                    metadata.kind
                },
                depth: self.scopes.len() + 1,
                ..metadata
            },
            ..Default::default()
        };

        self.scopes.push(Rc::new(RefCell::new(scope)));
        self.scopes.last().unwrap().clone()
    }

    pub fn leave_scope<F>(&mut self, mut callop: F)
    where
        F: FnMut(Rc<RefCell<Scope>>),
    {
        if let Some(symbol) = self.scopes.pop() {
            callop(symbol)
        }
    }

    pub fn bind(&mut self, sid: SymbolId, sym: Symbol) {
        if let Some(current) = self.scopes.last() {
            let current = current.borrow();
            let mut table = current.locals.borrow_mut();
            let shared_symbol = Rc::new(RefCell::new(sym));

            self.birdeye.insert(sid, Rc::clone(&shared_symbol));
            table.bind(sid, Rc::clone(&shared_symbol));
        }
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

pub struct SymbolTable {
    storage: HashMap<SymbolId, SharedSymbol>,
    field: HashMap<String, SymbolId>,
}

impl std::fmt::Debug for SymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("SymbolTable");
        for (id, sym) in &self.storage {
            let sym = sym.borrow();
            debug_struct.field(
                &format!("{} ({})", sym.name, id.0),
                &format!(
                    "{:?}{}",
                    sym.loc,
                    sym.signal
                        .map(|s| format!(" [{}]", s))
                        .unwrap_or(String::new())
                ),
            );
        }
        debug_struct.finish()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for SymbolTable {
    type Target = HashMap<SymbolId, SharedSymbol>;

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

    pub fn bind(&mut self, sid: SymbolId, sym: SharedSymbol) {
        let indent = sym.borrow().name.clone();
        self.storage.insert(sid, sym);
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
