use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref LABEL_TRACK: Mutex<u32> = Mutex::new(0);
}

#[derive(Debug, Clone)]
pub struct Label {
    id: u32,
    kind: LabelKind,
}

impl Label {
    pub const COND: &'static str = "cond";
    pub const LOOP: &'static str = "loop";

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
        let mut guard = LABEL_TRACK.lock().unwrap();
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

#[derive(Debug, Clone, strum_macros::Display)]
pub enum LabelKind {
    #[strum(to_string = "{0}")]
    Raw(String),
    #[strum(to_string = "ipt")]
    Ipt,
}
