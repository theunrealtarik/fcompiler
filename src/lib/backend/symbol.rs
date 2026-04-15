use super::mem::{Location, Register};
use super::tags::*;
use crate::frontend::ast::StatementContext;
use crate::game::SignalId;

#[allow(unused_imports)]
use crate::log;

use std::collections::HashMap;

pub type SharedScope = Scope;
pub type SharedSymbol = Symbol;

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

impl ScopeKind {
    pub fn is_breakable(&self) -> bool {
        matches!(self, ScopeKind::For | ScopeKind::While | ScopeKind::Loop)
    }

    pub fn is_continueable(&self) -> bool {
        matches!(self, ScopeKind::For | ScopeKind::While | ScopeKind::Loop)
    }

    pub fn contains_break(&self, stmts: &Vec<StatementContext>) -> bool {
        todo!()
    }
}

#[derive(Debug, Default, derive_builder::Builder, Clone)]
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

#[derive(Default, Clone)]
pub struct Scope {
    pub parent: Option<ScopeId>,
    pub children: Vec<ScopeId>,
    pub locals: SymbolTable,
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

#[derive(Debug, Default, Clone)]
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

    pub fn current(&self) -> Option<&Scope> {
        if let Some(idx) = self.stack.get(self.current - 1) {
            self.table.get(idx)
        } else {
            None
        }
    }

    pub fn current_mut(&mut self) -> Option<&mut SharedScope> {
        if let Some(idx) = self.stack.get(self.current - 1) {
            self.table.get_mut(idx)
        } else {
            None
        }
    }

    pub fn set_current(&mut self, idx: usize) {
        if idx >= self.table.len() {
            panic!("index out of bounds");
        }

        self.current = idx;
    }

    /// Resolve a symbol within a given scope based on its ident
    pub fn resolve(&self, scope_idx: ScopeId, name: &String) -> Option<SymbolHandle> {
        if let Some(scope) = self.table.get(&scope_idx)
            && let Some(sym_handle) = scope.locals.lookup_name(name)
        {
            return Some(sym_handle);
        }

        if let Some(scope) = self.table.get(&scope_idx)
            && let Some(parent) = scope.parent
        {
            return self.resolve(parent, name);
        }

        None
    }

    /// Uses the birdeye to get the symbol's scope which it uses to resolve data
    pub fn snatch(&self, sid: &SymbolId) -> Option<&SharedSymbol> {
        if let Some(scope_id) = self.birdeye.get(&sid)
            && let Some(scope) = self.table.get(&scope_id)
        {
            if let Some(sym) = scope.locals.get(&sid) {
                return Some(sym);
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

            match scope.locals.lookup_name(name) {
                Some(sym_handle) if &sym_handle.sym.name == name => {
                    return Some(sym_handle);
                }
                _ => {}
            }

            match scope.parent {
                Some(parent) if parent != idx => idx = parent,
                _ => break,
            }
        }

        None
    }

    pub fn define_symbol(&mut self, sid: SymbolId, sym: Symbol) {
        if let Some(scope) = self.current_mut() {
            let idx = scope.metadata.idx().clone();
            scope.locals.bind(sid, sym);
            self.birdeye.insert(sid, idx);
        }
    }

    pub fn enter_scope_explicit(
        &mut self,
        parent: Option<ScopeId>,
        metadata: ScopeMetadata,
    ) -> SharedScope {
        let idx = self.stack.len() + 1;
        self.current += 1;

        let depth = parent
            .map(|p| {
                self.table
                    .get(&p)
                    .map(|s| s.metadata.depth + 1)
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
        self.table.insert(idx, scope);

        if let Some(p) = parent {
            if let Some(scope) = self.table.get_mut(&p) {
                scope.children.push(idx);
            }
        }

        self.last().unwrap().clone()
    }

    pub fn enter_scope(&mut self, parent: Option<ScopeId>) -> SharedScope {
        self.enter_scope_explicit(parent, ScopeMetadata::default())
    }

    /// Pops the current scope from the stack, and readjusts the current scope index
    pub fn leave_current(&mut self) {
        self.stack.pop();
        self.current -= 1;
    }

    pub fn drop_scope(&mut self, scope_idx: &ScopeId) -> Option<SharedScope> {
        self.table.remove(scope_idx)
    }

    pub fn ladder(&self, current: &Scope) -> Vec<SharedScope> {
        let mut ladder = Vec::new();
        ladder.push(current.clone());

        if let Some(parent) = current.parent
            && let Some(parent) = self.table.get(&parent)
        {
            ladder.append(&mut self.ladder(parent));
        }

        ladder
    }
}

#[derive(Debug, Clone)]
pub struct SymbolHandle {
    pub sid: SymbolId,
    pub sym: SharedSymbol,
}

#[derive(Debug, Clone)]
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

#[derive(Clone)]
pub struct SymbolTable {
    storage: HashMap<SymbolId, SharedSymbol>,
    field: HashMap<String, SymbolId>,
}

impl std::fmt::Debug for SymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("SymbolTable");
        for (id, sym) in &self.storage {
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
        let indent = sym.name.clone();
        self.storage.insert(sid, sym);
        self.field.insert(indent, sid);
    }

    pub fn lookup_register(&self, reg: &Register) -> Option<&SharedSymbol> {
        self.storage
            .iter()
            .find(|(_, sym)| match sym.loc {
                Location::Reg(r) => &r == reg,
                _ => false,
            })
            .map(|(_, sym)| sym)
    }

    pub fn lookup_name(&self, name: &String) -> Option<SymbolHandle> {
        let sid = self.field.get(name)?;
        self.storage.get(sid).map(|sym| SymbolHandle {
            sid: *sid,
            sym: sym.clone(),
        })
    }
}
