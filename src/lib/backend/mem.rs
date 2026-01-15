// channeling

pub enum MemoryChannel {
    Chann1,
    Chann2,
    Chann3,
    Chann4,
}

// cpu registers

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Register(u8);

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

// register allocator

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

    pub unsafe fn alloc(&mut self) -> Register {
        let idx = self.free.trailing_zeros();
        let mask = !(1u64 << idx);
        self.free &= mask;

        Register(idx as u8)
    }

    pub unsafe fn free(&mut self, r: Register) {
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

// locations

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
            VariableLocation::STK(s) => s,
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

// output

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

// memory manager
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum Mark {
    Alive,
    Dead,
}

#[derive(Debug)]
pub struct MemoryManager {
    marks: HashMap<Register, Mark>,
    pub regs: RegisterAllocator,
    pub stack: StackAllocator,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            marks: HashMap::new(),
            regs: RegisterAllocator::new(),
            stack: StackAllocator::new(),
        }
    }

    pub fn free(&mut self, reg: Register) {
        if *reg >= RegisterAllocator::MAX_REGISTERS {
            return;
        }

        if !self.regs.is_used(reg) {
            return;
        }

        self.marks.insert(reg, Mark::Dead);
        unsafe {
            self.regs.free(reg);
            log::debug!("free( {:?} )", reg);
        }
    }

    pub fn alloc(&mut self) -> Result<Register, crate::error::CompileErrorKind> {
        if self.regs.free == 0 {
            return Err(crate::error::CompileErrorKind::Generation(
                crate::error::GeneratorError::OutOfRegisters,
            ));
        }

        let reg = unsafe { self.regs.alloc() };
        self.marks.insert(reg, Mark::Alive);
        Ok(reg)
    }

    pub fn live_marks(&self) -> Vec<Register> {
        self.marks
            .iter()
            .filter(|(_, m)| *m == &Mark::Alive)
            .map(|(r, _)| *r)
            .collect::<Vec<Register>>()
    }

    pub fn marks(&self) -> &HashMap<Register, Mark> {
        &self.marks
    }

    pub fn dead_marks(&self) -> Vec<Register> {
        self.marks
            .iter()
            .filter(|(_, m)| *m == &Mark::Dead)
            .map(|(r, _)| *r)
            .collect::<Vec<Register>>()
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

