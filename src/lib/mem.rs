// cpu registers
#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Register {
    #[strum(serialize = "reg1")]
    R1 = 1,
    #[strum(serialize = "reg2")]
    R2 = 2,
    #[strum(serialize = "reg3")]
    R3 = 3,
    #[strum(serialize = "reg4")]
    R4 = 4,
    #[strum(serialize = "reg5")]
    R5 = 5,
    #[strum(serialize = "reg6")]
    R6 = 6,
    #[strum(serialize = "reg7")]
    R7 = 7,
    #[strum(serialize = "reg8")]
    R8 = 8,
    #[strum(serialize = "reg9")]
    R9 = 9,
}

impl TryFrom<u8> for Register {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Register::R1),
            2 => Ok(Register::R2),
            3 => Ok(Register::R3),
            4 => Ok(Register::R4),
            5 => Ok(Register::R5),
            6 => Ok(Register::R6),
            7 => Ok(Register::R7),
            8 => Ok(Register::R8),
            9 => Ok(Register::R9),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct RegisterAllocator {
    free: Vec<Register>,
}

impl Default for RegisterAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl RegisterAllocator {
    pub fn new() -> Self {
        Self {
            free: vec![
                Register::R9,
                Register::R8,
                Register::R7,
                Register::R6,
                Register::R5,
                Register::R4,
                Register::R3,
                Register::R2,
                Register::R1,
            ],
        }
    }

    pub fn alloc(&mut self) -> Option<Register> {
        self.free.pop()
    }

    pub fn free(&mut self, r: Register) {
        if !self.free.contains(&r) {
            self.free.push(r);
        }
    }
}

// stack
#[derive(Debug, Clone)]
pub struct StackAllocator {
    elements: Vec<u8>,
}

impl StackAllocator {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn push(&mut self, r: u8) {
        self.elements.push(r)
    }

    pub fn pop(&mut self) -> Option<u8> {
        self.elements.pop()
    }

    pub fn peek(&self) -> Option<&u8> {
        self.elements.last()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn size(&self) -> usize {
        self.elements.len()
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }
}

impl Default for StackAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// channeling
pub enum MemoryChannel {
    Chann1,
    Chann2,
    Chann3,
    Chann4,
}

// variable
#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub reg: Register,
    pub slot: Option<u8>,
    pub value: Option<i32>,
    pub signal: Option<crate::game::Signal>,
}

impl Variable {
    pub fn new(
        name: String,
        reg: Register,
        slot: Option<u8>,
        value: Option<i32>,
        signal: Option<crate::game::Signal>,
    ) -> Self {
        Self {
            name,
            reg,
            slot,
            value,
            signal,
        }
    }
}

// out
#[derive(Debug, Clone, Copy)]
pub struct Out(pub Register);

impl std::ops::Deref for Out {
    type Target = Register;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Out {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for Out {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "out{}", self.0 as u8 + 1)
    }
}
