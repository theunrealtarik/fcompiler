use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref LABEL_TRACK: Mutex<u32> = Mutex::new(0);
}

#[derive(Debug, Clone)]
pub struct Label(String);

impl Label {
    pub fn from<T>(src: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self(format!("{}", src))
    }

    pub fn fresh<T>(src: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self(format!("{}_{}", src, Self::id()))
    }

    pub fn unique<T>(src: T, id: u32) -> Self
    where
        T: std::fmt::Display,
    {
        Self(format!("{}_{}", src, id))
    }

    pub fn id() -> u32 {
        let mut guard = LABEL_TRACK.lock().unwrap();
        let id = *guard;
        *guard += 1;
        id
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for Label {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
