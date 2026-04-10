use std::sync::Mutex;

pub type ScopeId = usize;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TempId(pub i32);

impl std::ops::Deref for TempId {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
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

lazy_static::lazy_static! {
    static ref TEMP_ID_TRACK: Mutex<i32> = Mutex::new(0);
    static ref SYMB_ID_TRACK: Mutex<i32> = Mutex::new(0);
    static ref LBEL_ID_TRACK: Mutex<u32> = Mutex::new(0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::EnumIs)]
pub enum Operand {
    Persistent(SymbolId),
    Temp(TempId),
    Imm(i32),
}

#[allow(clippy::from_over_into)]
impl Into<i32> for Operand {
    fn into(self) -> i32 {
        match self {
            Operand::Persistent(pid) => *pid,
            Operand::Temp(tid) => *tid,
            Operand::Imm(n) => n,
        }
    }
}

impl Operand {
    pub fn temp() -> Operand {
        let mut guard = TEMP_ID_TRACK.lock().unwrap();
        let id = *guard;
        *guard += 1;
        Operand::Temp(TempId(id))
    }

    pub fn persistent() -> Operand {
        let mut guard = SYMB_ID_TRACK.lock().unwrap();
        let id = *guard;
        *guard += 1;
        Operand::Persistent(SymbolId(id))
    }

    pub fn immediate(value: i32) -> Operand {
        Operand::Imm(value)
    }
}

#[derive(Debug, Clone)]
pub struct Label {
    id: u32,
    kind: LabelKind,
}

#[derive(Debug, Clone, strum_macros::Display)]
pub enum LabelKind {
    #[strum(to_string = "{0}")]
    Raw(String),
    #[strum(to_string = "ipt")]
    Ipt,
}

impl Label {
    pub const COND: &'static str = "cond";
    pub const LOOP: &'static str = "loop";
    pub const WHILE: &'static str = "while";

    pub fn new(kind: LabelKind) -> Self {
        let id = Self::id();
        Self { id, kind }
    }

    pub fn raw(string: &str) -> Self {
        Self {
            id: Self::id(),
            kind: LabelKind::Raw(Self::sanitize(string)),
        }
    }

    pub fn suffix(&self, suffix: &str) -> Label {
        match &self.kind {
            LabelKind::Raw(s) => Label {
                id: self.id,
                kind: LabelKind::Raw(format!("{}_{}", s, Self::sanitize(suffix))),
            },
            _ => unreachable!(),
        }
    }

    fn sanitize(string: &str) -> String {
        string.replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
    }

    fn id() -> u32 {
        let mut guard = LBEL_ID_TRACK.lock().unwrap();
        let id = *guard;
        *guard += 1;
        id
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            LabelKind::Raw(s) => write!(f, ":{}_{}", s, self.id),
            label => write!(f, "{}", label),
        }
    }
}
