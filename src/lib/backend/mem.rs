use crate::error::*;
use std::collections::HashMap;

// channeling

pub enum MemoryChannel {
    Chann1,
    Chann2,
    Chann3,
    Chann4,
}

// cpu registers

pub const MAX_REGISTERS: u8 = 63;

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

impl TryFrom<i32> for Register {
    type Error = CompileErrorKind;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value >= 0 && value < MAX_REGISTERS as i32 {
            Ok(Register(value as u8))
        } else {
            Err(CompileErrorKind::Generation(
                GeneratorError::InvalidRegister,
            ))
        }
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

        for b in 0..MAX_REGISTERS {
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
        if value > MAX_REGISTERS {
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
pub enum Location {
    Reg(Register),
    Stk(StackSlot),
}

#[allow(clippy::from_over_into)]
impl Into<Register> for Location {
    fn into(self) -> Register {
        match self {
            Location::Reg(r) => r,
            Location::Stk(s) => *s,
        }
    }
}

impl Location {
    pub fn as_register(&self) -> &Register {
        match self {
            Location::Reg(r) => r,
            Location::Stk(s) => s,
        }
    }
}

#[derive(Debug, strum_macros::EnumIs)]
pub enum Resolved {
    Reg(Register),
    Imm(i32),
}

impl std::fmt::Display for Resolved {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Resolved::Reg(r) => write!(f, "{}", r),
            Resolved::Imm(n) => write!(f, "{}", n),
        }
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

#[derive(Debug, PartialEq, Eq)]
pub enum Mark {
    Alive,
    Dead,
}

#[derive(Debug, Default)]
pub struct MemoryManager {
    marks: HashMap<Register, Mark>,
    pub temps: HashMap<super::asm::TempId, Register>,
    pub regs: RegisterAllocator,
    pub stack: StackAllocator,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn free(&mut self, reg: Register) {
        if *reg >= MAX_REGISTERS {
            panic!("attempted to free a non-allocatable register {:?}", reg);
        }

        if !self.regs.is_used(reg) {
            panic!("double or invalid free of {:?}", reg);
        }

        self.marks.insert(reg, Mark::Dead);
        unsafe {
            self.regs.free(reg);
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
