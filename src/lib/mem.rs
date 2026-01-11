use std::collections::HashSet;

// cpu registers (64 registers)
static MAX_REGISTERS: u8 = 63;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Register(u8);

impl TryFrom<u8> for Register {
    type Error = crate::error::CompileError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > MAX_REGISTERS {
            return Err(crate::error::CompileError::RegistersExceedLimit);
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
    pub fn new() -> Self {
        Self { free: u64::MAX }
    }

    pub fn alloc(&mut self) -> Result<Register, crate::error::CompileError> {
        if self.free == 0 {
            return Err(crate::error::CompileError::OutOfRegisters);
        }

        let idx = self.free.trailing_zeros();
        let mask = !(1u64 << idx);
        self.free &= mask;
        Ok(Register(idx as u8))
    }

    pub fn free(&mut self, r: Register) {
        if r >= MAX_REGISTERS {
            return;
        }

        let mask = 1u64 << r;
        self.free |= mask;
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
