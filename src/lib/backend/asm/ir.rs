use lazy_static::lazy_static;
use std::sync::Mutex;
use std::vec;

lazy_static! {
    static ref TEMP_ID_TRACK: Mutex<i32> = Mutex::new(0);
    static ref SYMB_ID_TRACK: Mutex<i32> = Mutex::new(0);
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TempId(pub i32);

impl std::ops::Deref for TempId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::EnumIs)]
pub enum Operand {
    Persistent(crate::backend::symbol::SymbolId),
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

use crate::backend::asm::Label;
use crate::backend::symbol::*;
use crate::frontend::ast::*;

#[derive(Debug, Clone)]
pub enum Instruction {
    BinOp {
        dst: Operand,
        lhs: Operand,
        rhs: Operand,
        op: BinOp,
    },

    UnaryOp {
        dst: Operand,
        src: Operand,
        op: UnaryOp,
    },

    Inc {
        dst: Operand,
    },

    Dec {
        dst: Operand,
    },

    Mov {
        dst: Operand,
        src: Operand,
    },

    MovSig {
        dst: Operand,
        src: Operand,
        signal_id: crate::game::SignalId,
    },

    Out {
        src: Operand,
        signal_id: Option<crate::game::SignalId>,
    },

    Compare {
        a: Operand,
        b: Operand,
        op: CmpOp,
        addr: Option<Label>,
    },

    TestType {
        a: Operand,
        b: Operand,
    },

    Label {
        name: String,
    },

    Jump {
        addr: Label,
        offset: Option<u8>,
    },

    Nop,
}

impl Instruction {
    pub fn sources(&self) -> Vec<&Operand> {
        match self {
            Self::BinOp { lhs, rhs, .. } => vec![lhs, rhs],
            Self::UnaryOp { src, .. } => vec![src],
            Self::Mov { src, .. } => vec![src],
            Self::MovSig { src, .. } => vec![src],
            Self::Out { src, .. } => vec![src],
            Self::Compare { a, b, .. } => vec![a, b],
            Self::TestType { a, b, .. } => vec![a, b],
            _ => vec![],
        }
    }
}

/// ir emitting helpers
impl super::Assembler {
    /// mov: dst = src
    pub fn mov(&mut self, dst: Operand, src: Operand) {
        self.instr.push(Instruction::Mov { dst, src });
    }

    /// add: dst = lhs + rhs
    pub fn add(&mut self, dst: Operand, lhs: Operand, rhs: Operand) {
        self.instr.push(Instruction::BinOp {
            dst,
            lhs,
            rhs,
            op: BinOp::Add,
        });
    }

    /// inc: dst += 1
    pub fn inc(&mut self, dst: Operand) {
        self.instr.push(Instruction::Inc { dst })
    }

    /// dec: dst -= 1
    pub fn dec(&mut self, dst: Operand) {
        self.instr.push(Instruction::Dec { dst })
    }

    /// sub: dst = lhs - rhs
    pub fn sub(&mut self, dst: Operand, lhs: Operand, rhs: Operand) {
        self.instr.push(Instruction::BinOp {
            dst,
            lhs,
            rhs,
            op: BinOp::Sub,
        });
    }

    /// mul: dst = lhs * rhs
    pub fn mul(&mut self, dst: Operand, lhs: Operand, rhs: Operand) {
        self.instr.push(Instruction::BinOp {
            dst,
            lhs,
            rhs,
            op: BinOp::Mul,
        });
    }

    /// div: dst = lhs / rhs
    pub fn div(&mut self, dst: Operand, lhs: Operand, rhs: Operand) {
        self.instr.push(Instruction::BinOp {
            dst,
            lhs,
            rhs,
            op: BinOp::Div,
        });
    }

    /// mod: dst = lhs % rhs
    pub fn modu(&mut self, dst: Operand, lhs: Operand, rhs: Operand) {
        self.instr.push(Instruction::BinOp {
            dst,
            lhs,
            rhs,
            op: BinOp::Mod,
        });
    }

    /// unary not: dst = !src
    pub fn not(&mut self, dst: Operand, src: Operand) {
        self.instr.push(Instruction::UnaryOp {
            dst,
            src,
            op: UnaryOp::Not,
        });
    }

    /// unary neg: dst = -src
    pub fn neg(&mut self, dst: Operand, src: Operand) {
        self.instr.push(Instruction::UnaryOp {
            dst,
            src,
            op: UnaryOp::Neg,
        });
    }

    /// output instruction: send src to signal
    pub fn out(&mut self, src: Operand, signal_id: Option<crate::game::SignalId>) {
        self.instr.push(Instruction::Out { src, signal_id });
    }

    /// move signal
    pub fn mov_sig(&mut self, dst: Operand, src: Operand, signal_id: crate::game::SignalId) {
        self.instr.push(Instruction::MovSig {
            dst,
            src,
            signal_id,
        });
    }

    /// generic compare
    pub fn compare(&mut self, a: Operand, b: Operand, op: CmpOp, addr: Option<Label>) {
        self.instr.push(Instruction::Compare { a, b, op, addr });
    }

    /// test value comparison
    pub fn test_val(&mut self, a: Operand, b: Operand, op: CmpOp) {
        self.instr.push(Instruction::Compare {
            a,
            b,
            op,
            addr: None,
        });
    }

    /// test type comparison
    pub fn test_type(&mut self, a: Operand, b: Operand) {
        self.instr.push(Instruction::TestType { a, b });
    }

    /// `a == b`
    pub fn test_eq(&mut self, a: Operand, b: Operand) {
        self.test_val(a, b, CmpOp::Eq);
    }

    /// `a != b`
    pub fn test_ne(&mut self, a: Operand, b: Operand) {
        self.test_val(a, b, CmpOp::Ne);
    }

    /// `a < b`
    pub fn test_lt(&mut self, a: Operand, b: Operand) {
        self.test_val(a, b, CmpOp::Lt);
    }

    /// `a > b`
    pub fn test_gt(&mut self, a: Operand, b: Operand) {
        self.test_val(a, b, CmpOp::Gt);
    }

    /// `a <= b`
    pub fn test_le(&mut self, a: Operand, b: Operand) {
        self.test_val(a, b, CmpOp::Le);
    }

    /// `a >= b`
    pub fn test_ge(&mut self, a: Operand, b: Operand) {
        self.test_val(a, b, CmpOp::Ge);
    }

    /// branch if `a == b`
    pub fn br_eq(&mut self, a: Operand, b: Operand, addr: Label) {
        self.compare(a, b, CmpOp::Eq, Some(addr));
    }

    /// branch if `a != b`
    pub fn br_ne(&mut self, a: Operand, b: Operand, addr: Label) {
        self.compare(a, b, CmpOp::Ne, Some(addr));
    }

    /// branch if `a < b`
    pub fn br_lt(&mut self, a: Operand, b: Operand, addr: Label) {
        self.compare(a, b, CmpOp::Lt, Some(addr));
    }

    /// branch if `a > b`
    pub fn br_gt(&mut self, a: Operand, b: Operand, addr: Label) {
        self.compare(a, b, CmpOp::Gt, Some(addr));
    }

    /// branch if `a <= b`
    pub fn br_le(&mut self, a: Operand, b: Operand, addr: Label) {
        self.compare(a, b, CmpOp::Le, Some(addr));
    }

    /// branch if `a >= b`
    pub fn br_ge(&mut self, a: Operand, b: Operand, addr: Label) {
        self.compare(a, b, CmpOp::Ge, Some(addr));
    }

    /// push a nop
    pub fn nop(&mut self) {
        self.instr.push(Instruction::Nop);
    }

    /// jump
    pub fn jump(&mut self, addr: Label, offset: Option<u8>) {
        self.instr.push(Instruction::Jump { addr, offset })
    }

    /// label
    pub fn label(&mut self, addr: Label) {
        self.instr.push(Instruction::Label {
            name: addr.to_string(),
        })
    }
}
