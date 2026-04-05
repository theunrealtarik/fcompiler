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
            .field("locals", &self.locals)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct ScopeArena {
    stack: Vec<ScopeId>,
    table: HashMap<ScopeId, SharedScope>,
    birdeye: HashMap<SymbolId, ScopeId>,
    current: usize,
}

impl ScopeArena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, scope_idx: ScopeId) -> Option<&SharedScope> {
        self.table.get(&scope_idx)
    }

    pub fn last(&self) -> Option<&SharedScope> {
        self.stack.last().and_then(|id| self.table.get(id))
    }

    pub fn current(&self) -> Option<SharedScope> {
        self.table.get(&self.current).cloned()
    }

    // resolve a symbol within a given scope based on its ident
    pub fn resolve(&self, scope_idx: ScopeId, name: &String) -> Option<SymbolHandle> {
        if let Some(scope) = self.table.get(&scope_idx)
            && let Some(sym_handle) = scope.borrow().locals.borrow().lookup_name(name)
        {
            return Some(sym_handle);
        }

        if let Some(scope) = self.table.get(&scope_idx)
            && let Some(parent) = scope.borrow().parent
        {
            return self.resolve(parent, name);
        }

        None
    }

    // uses the birdeye to get the symbol's scope which it uses to resolve data
    pub fn snatch(&self, sid: &SymbolId) -> Option<SharedSymbol> {
        if let Some(scope_id) = self.birdeye.get(&sid)
            && let Some(scope) = self.table.get(&scope_id)
        {
            let scope = scope.borrow();
            let table = scope.locals.borrow();

            if let Some(sym) = table.get(&sid) {
                return Some(Rc::clone(sym));
            }
        }

        None
    }

    pub fn lookup(&self, name: &String) -> Option<SymbolHandle> {
        let mut idx = *self
            .stack
            .last()
            .expect("looking up a symbol requires a global scope");

        loop {
            let scope = match self.table.get(&idx) {
                Some(s) => s,
                None => break,
            };

            let scope_ref = scope.borrow();
            let table = scope_ref.locals.borrow();

            match table.lookup_name(name) {
                Some(sym_handle) if &sym_handle.sym.borrow().name == name => {
                    return Some(sym_handle);
                }
                _ => {}
            }

            match scope_ref.parent {
                Some(parent) if parent != idx => idx = parent,
                _ => break,
            }
        }

        None
    }

    pub fn define_symbol(&mut self, sid: SymbolId, sym: Symbol) {
        if let Some(scope) = self.current() {
            let shared_symbol = Rc::new(RefCell::new(sym));
            let scope = scope.borrow();
            let mut table = scope.locals.borrow_mut();

            self.birdeye.insert(sid, scope.metadata.idx());
            table.bind(sid, Rc::clone(&shared_symbol));
        }
    }

    pub fn enter_scope_explicit(
        &mut self,
        parent: Option<ScopeId>,
        metadata: ScopeMetadata,
    ) -> SharedScope {
        let idx = self.stack.len();
        self.current = idx;

        if let Some(p) = parent {
            if let Some(scope_rc) = self.table.get(&p) {
                // scope_rc.borrow_mut().children.push(id);
            }
        }

        let depth = parent
            .map(|p| {
                self.table
                    .get(&p)
                    .map(|s| s.borrow().metadata.depth + 1)
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        let scope = Scope {
            parent,
            metadata: ScopeMetadata {
                idx,
                kind: if self.stack.is_empty() {
                    ScopeKind::Global
                } else {
                    metadata.kind
                },
                depth,
                ..metadata
            },
            ..Default::default()
        };

        self.stack.push(idx);
        self.table.insert(idx, Rc::new(RefCell::new(scope)));
        self.last().unwrap().clone()
    }

    pub fn enter_scope(&mut self, parent: Option<ScopeId>) -> SharedScope {
        self.enter_scope_explicit(parent, ScopeMetadata::default())
    }

    pub fn leave_scope(&mut self) {
        self.current -= 1;
    }

    pub fn drop_scope(&mut self, scope_idx: &ScopeId) -> Option<SharedScope> {
        self.table.remove(scope_idx)
    }

    pub fn ladder(&self, ground: ScopeId) -> Vec<SharedScope> {
        self.stack[0..=ground]
            .iter()
            .rev()
            .collect::<Vec<&ScopeId>>()
            .into_iter()
            .filter_map(|scope_idx| self.table.get(scope_idx))
            .cloned()
            .collect::<Vec<_>>()
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
