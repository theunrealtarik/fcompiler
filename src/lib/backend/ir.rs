// operands
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref TEMP_ID_TRACK: Mutex<i32> = Mutex::new(0);
    static ref SYMB_ID_TRACK: Mutex<i32> = Mutex::new(0);
}

type SymbolId = i32;
type TempId = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Persistent(SymbolId),
    Temp(TempId),
    Imm(i32),
}

#[allow(clippy::from_over_into)]
impl Into<i32> for Operand {
    fn into(self) -> i32 {
        match self {
            Operand::Persistent(pid) => pid,
            Operand::Temp(tid) => tid,
            Operand::Imm(n) => n,
        }
    }
}

impl Operand {
    pub fn temp() -> Operand {
        let mut guard = TEMP_ID_TRACK.lock().unwrap();
        let id = *guard;
        *guard += 1;
        Operand::Temp(id)
    }

    pub fn persistent() -> Operand {
        let mut guard = SYMB_ID_TRACK.lock().unwrap();
        let id = *guard;
        *guard += 1;
        Operand::Persistent(id)
    }

    pub fn immediate(value: i32) -> Operand {
        Operand::Imm(value)
    }
}

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
        signal_id: Option<String>,
    },

    Out {
        src: Operand,
        signal_id: Option<String>,
    },

    Nop,
}
