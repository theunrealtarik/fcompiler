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
    #[strum(serialize = "reg10")]
    R10 = 10,
    #[strum(serialize = "reg11")]
    R11 = 11,
    #[strum(serialize = "reg12")]
    R12 = 12,
    #[strum(serialize = "reg13")]
    R13 = 13,
    #[strum(serialize = "reg14")]
    R14 = 14,
    #[strum(serialize = "reg15")]
    R15 = 15,
    #[strum(serialize = "reg16")]
    R16 = 16,
    #[strum(serialize = "reg17")]
    R17 = 17,
    #[strum(serialize = "reg18")]
    R18 = 18,
    #[strum(serialize = "reg19")]
    R19 = 19,
    #[strum(serialize = "reg20")]
    R20 = 20,
    #[strum(serialize = "reg21")]
    R21 = 21,
    #[strum(serialize = "reg22")]
    R22 = 22,
    #[strum(serialize = "reg23")]
    R23 = 23,
    #[strum(serialize = "reg24")]
    R24 = 24,
    #[strum(serialize = "reg25")]
    R25 = 25,
    #[strum(serialize = "reg26")]
    R26 = 26,
    #[strum(serialize = "reg27")]
    R27 = 27,
    #[strum(serialize = "reg28")]
    R28 = 28,
    #[strum(serialize = "reg29")]
    R29 = 29,
    #[strum(serialize = "reg30")]
    R30 = 30,
    #[strum(serialize = "reg31")]
    R31 = 31,
    #[strum(serialize = "reg32")]
    R32 = 32,
    #[strum(serialize = "reg33")]
    R33 = 33,
    #[strum(serialize = "reg34")]
    R34 = 34,
    #[strum(serialize = "reg35")]
    R35 = 35,
    #[strum(serialize = "reg36")]
    R36 = 36,
    #[strum(serialize = "reg37")]
    R37 = 37,
    #[strum(serialize = "reg38")]
    R38 = 38,
    #[strum(serialize = "reg39")]
    R39 = 39,
    #[strum(serialize = "reg40")]
    R40 = 40,
    #[strum(serialize = "reg41")]
    R41 = 41,
    #[strum(serialize = "reg42")]
    R42 = 42,
    #[strum(serialize = "reg43")]
    R43 = 43,
    #[strum(serialize = "reg44")]
    R44 = 44,
    #[strum(serialize = "reg45")]
    R45 = 45,
    #[strum(serialize = "reg46")]
    R46 = 46,
    #[strum(serialize = "reg47")]
    R47 = 47,
    #[strum(serialize = "reg48")]
    R48 = 48,
    #[strum(serialize = "reg49")]
    R49 = 49,
    #[strum(serialize = "reg50")]
    R50 = 50,
    #[strum(serialize = "reg51")]
    R51 = 51,
    #[strum(serialize = "reg52")]
    R52 = 52,
    #[strum(serialize = "reg53")]
    R53 = 53,
    #[strum(serialize = "reg54")]
    R54 = 54,
    #[strum(serialize = "reg55")]
    R55 = 55,
    #[strum(serialize = "reg56")]
    R56 = 56,
    #[strum(serialize = "reg57")]
    R57 = 57,
    #[strum(serialize = "reg58")]
    R58 = 58,
    #[strum(serialize = "reg59")]
    R59 = 59,
    #[strum(serialize = "reg60")]
    R60 = 60,
    #[strum(serialize = "reg61")]
    R61 = 61,
    #[strum(serialize = "reg62")]
    R62 = 62,
    #[strum(serialize = "reg63")]
    R63 = 63,
    #[strum(serialize = "reg64")]
    R64 = 64,
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
    pub signal: Option<crate::game::SignalId>,
}

impl Variable {
    pub fn new(
        name: String,
        reg: Register,
        slot: Option<u8>,
        signal: Option<crate::game::SignalId>,
    ) -> Self {
        Self {
            name,
            reg,
            slot,
            signal,
        }
    }
}

// out
#[derive(Debug, Clone, Copy)]
pub struct Out(pub u8);

impl std::ops::Deref for Out {
    type Target = u8;

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
        write!(f, "out{}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct OutputManager {
    last: u8,
}

impl OutputManager {
    pub fn out(&mut self) -> Out {
        self.last += 1;
        Out(self.last)
    }
}
