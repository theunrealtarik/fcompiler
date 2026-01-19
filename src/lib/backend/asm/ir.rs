use lazy_static::lazy_static;
use std::sync::Mutex;

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
    Persistent(super::super::symbol::SymbolId),
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

    Nop,
}

impl Instruction {
    pub fn sources(&self) -> Vec<&Operand> {
        match self {
            Instruction::BinOp { lhs, rhs, .. } => vec![lhs, rhs],
            Instruction::UnaryOp { src, .. } => vec![src],
            Instruction::Mov { src, .. } => vec![src],
            Instruction::MovSig { src, .. } => vec![src],
            Instruction::Out { src, .. } => vec![src],
            Instruction::Nop => vec![],
        }
    }
}
