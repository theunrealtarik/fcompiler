use super::mem::Location;
use crate::{
    backend::{asm::Label, mem::Register},
    game::SignalId,
    log,
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
    Function,
}

#[derive(Debug, Default, derive_builder::Builder)]
pub struct ScopeMetadata {
    #[builder(setter(skip = true))]
    idx: usize,
    #[builder(setter(skip = true))]
    depth: usize,
    pub kind: ScopeKind,
    #[builder(setter(into, strip_option), default = None)]
    pub start_label: Option<Label>,
    #[builder(setter(into, strip_option), default = None)]
    pub exit_label: Option<Label>,
}

impl ScopeMetadata {
    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}

pub type ScopeId = usize;
pub type SharedScope = Rc<RefCell<Scope>>;

#[derive(Default)]
pub struct Scope {
    pub parent: Option<ScopeId>,
    pub children: Vec<ScopeId>,
    pub locals: RefCell<SymbolTable>,
    pub metadata: ScopeMetadata,
}

impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("{:?} Scope", self.metadata.kind))
            .field("parent", &self.parent)
            .field("children", &self.children)
            .field("metadata", &self.metadata)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct ScopeArena {
    scopes: Vec<SharedScope>,
    birdeye: HashMap<SymbolId, ScopeId>,
}

impl ScopeArena {
    pub fn new() -> Self {
        Self::default()
    }

    // resolve a symbol within a given scope based on its ident
    pub fn resolve(&self, scope_idx: ScopeId, name: &String) -> Option<SymbolHandle> {
        if let Some(scope) = self.scopes.get(scope_idx)
            && let Some(sym_handle) = scope.borrow().locals.borrow().lookup_name(name)
        {
            return Some(sym_handle);
        }

        if let Some(parent) = self.scopes[scope_idx].borrow().parent {
            return self.resolve(parent, name);
        }

        None
    }

    // uses the birdeye to get the symbol's scope which it uses to resolve data
    pub fn snatch(&self, sid: &SymbolId) -> Option<SharedSymbol> {
        if let Some(scope_id) = self.birdeye.get(&sid)
            && let Some(scope) = self.scopes.get(*scope_id)
        {
            let scope = scope.borrow();
            let table = scope.locals.borrow();

            if let Some(sym) = table.get(&sid) {
                return Some(Rc::clone(sym));
            }
        }

        None
    }

    // looks up for a symbol based on its ident upward the global scope
    pub fn lookup(&self, name: &String) -> Option<SymbolHandle> {
        let current = self
            .scopes
            .last()
            .expect("looking up a symbol requires a global scope")
            .borrow();

        for scope in self.ladder(current.metadata.idx()) {
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

    pub fn define_symbol(&mut self, sid: SymbolId, sym: Symbol) {
        if let Some(current) = self.scopes.last() {
            let current = current.borrow();
            let mut table = current.locals.borrow_mut();
            let shared_symbol = Rc::new(RefCell::new(sym));

            self.birdeye.insert(sid, current.metadata.idx());
            table.bind(sid, Rc::clone(&shared_symbol));
        }
    }

    pub fn enter_scope_explicit(
        &mut self,
        parent: Option<ScopeId>,
        metadata: ScopeMetadata,
    ) -> SharedScope {
        let id = self.scopes.len();

        if let Some(p) = parent
            && let Some(scope) = self.scopes.get(p)
        {
            // scope.borrow_mut().children.push(id)
        }

        let scope = Scope {
            parent,
            metadata: ScopeMetadata {
                idx: id,
                kind: if self.scopes.is_empty() {
                    ScopeKind::Global
                } else {
                    metadata.kind
                },
                depth: parent
                    .map(|p| self.scopes[p].borrow().metadata.depth + 1)
                    .unwrap_or(0),
                ..metadata
            },
            ..Default::default()
        };

        log::debug!(" {:?}:{}", scope.metadata.kind, scope.metadata.depth);
        self.scopes.push(Rc::new(RefCell::new(scope)));
        self.scopes.last().unwrap().clone()
    }

    pub fn enter_scope(&mut self, parent: Option<ScopeId>) -> SharedScope {
        self.enter_scope_explicit(parent, ScopeMetadata::default())
    }

    pub fn ladder(&self, ground: ScopeId) -> Vec<SharedScope> {
        self.scopes[0..=ground].iter().rev().cloned().collect::<_>()
    }

    pub fn leave_scope<F>(&mut self, mut callop: F)
    where
        F: FnMut(SharedScope),
    {
        if let Some(symbol) = self.scopes.pop() {
            callop(symbol)
        }
    }
}

pub struct ScopeCursor {}

#[derive(Debug, Clone)]
pub struct SymbolHandle {
    pub sid: SymbolId,
    pub sym: SharedSymbol,
}

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub loc: Location,
    pub signal: Option<SignalId>,
}

impl Symbol {
    pub fn new(name: String, loc: Location, signal: Option<SignalId>) -> Self {
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
