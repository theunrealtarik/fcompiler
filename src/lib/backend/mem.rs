use log::debug;

// channeling
pub enum MemoryChannel {
    Chann1,
    Chann2,
    Chann3,
    Chann4,
}

// cpu registers (64 registers)
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Register(u8);

impl TryFrom<u8> for Register {
    type Error = crate::error::CompileError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > RegisterAllocator::MAX_REGISTERS {
            return Err(crate::error::CompileError::new(
                crate::error::CompileErrorKind::Generation(
                    crate::error::GeneratorError::InvalidRegister,
                ),
                None,
            ));
        }

        Ok(Self(value))
    }
}

impl std::ops::Deref for Register {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.0 + 1)
    }
}

#[derive(Debug)]
pub struct RegisterAllocator {
    free: u64,
}

impl Default for RegisterAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl RegisterAllocator {
    pub const MAX_REGISTERS: u8 = 63;

    pub fn new() -> Self {
        Self { free: u64::MAX }
    }

    pub fn alloc(&mut self) -> Result<Register, crate::error::CompileError> {
        if self.free == 0 {
            return Err(crate::error::CompileError::new(
                crate::error::CompileErrorKind::Generation(
                    crate::error::GeneratorError::OutOfRegisters,
                ),
                None,
            ));
        }

        let idx = self.free.trailing_zeros();
        let mask = !(1u64 << idx);
        self.free &= mask;

        let r = Register(idx as u8);
        Ok(r)
    }

    pub fn free(&mut self, r: Register) {
        debug!("free {:?}", r);
        if *r >= Self::MAX_REGISTERS {
            return;
        }

        if !self.is_used(r) {
            return;
        }

        let mask = 1u64 << *r;
        self.free |= mask;
    }

    pub fn occupied(&self) -> Vec<Register> {
        let mut occupied = Vec::new();

        for b in 0..Self::MAX_REGISTERS {
            if self.free & (1 << b) == 0 {
                occupied.push(Register(b));
            }
        }

        occupied
    }

    pub fn is_used(&self, r: Register) -> bool {
        self.free & (1 << *r) == 0
    }

    pub fn is_free(&self, r: Register) -> bool {
        self.free & (1 << *r) != 0
    }

    pub fn free_regs(&self) -> u32 {
        self.free.count_ones()
    }

    pub fn used_regs(&self) -> u32 {
        self.free.count_ones()
    }
}

// stack
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackSlot(Register);

impl std::ops::Deref for StackSlot {
    type Target = Register;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for StackSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct StackAllocator {
    elements: Vec<StackSlot>,
}

impl StackAllocator {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn push(&mut self, r: StackSlot) {
        self.elements.push(r)
    }

    pub fn pop(&mut self) -> Option<StackSlot> {
        self.elements.pop()
    }

    pub fn peek(&self) -> Option<&StackSlot> {
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

// location
#[derive(Debug, Clone, Copy, strum_macros::Display)]
pub enum VariableLocation {
    REG(Register),
    STK(StackSlot),
}

#[allow(clippy::from_over_into)]
impl Into<Register> for VariableLocation {
    fn into(self) -> Register {
        match self {
            VariableLocation::REG(r) => r,
            VariableLocation::STK(s) => *s,
        }
    }
}

impl VariableLocation {
    pub fn as_register(&self) -> &Register {
        match self {
            VariableLocation::REG(r) => r,
            VariableLocation::STK(s) => &*s,
        }
    }
}

#[derive(Debug, Clone, Copy, strum_macros::Display, strum_macros::EnumIs)]
pub enum OperandLocation {
    REG(Register),
    STK(StackSlot),
    IMM(i32),
}

impl From<VariableLocation> for OperandLocation {
    fn from(value: VariableLocation) -> Self {
        match value {
            VariableLocation::REG(r) => OperandLocation::REG(r),
            VariableLocation::STK(s) => OperandLocation::STK(s),
        }
    }
}

// variable
#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub loc: VariableLocation,
    pub signal: Option<crate::game::SignalId>,
}

impl Variable {
    pub fn new(name: String, loc: VariableLocation, signal: Option<crate::game::SignalId>) -> Self {
        Self { name, loc, signal }
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
