use super::ir::*;
use super::mem::*;

use crate::frontend::ast::*;

use log::debug;

#[derive(Debug)]
pub struct AssemblyEmitter {
    code: String,
}

impl Default for AssemblyEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl AssemblyEmitter {
    pub fn new() -> Self {
        Self {
            code: String::from("clr\n"),
        }
    }

    pub fn mov<D, S>(&mut self, dst: D, src: S)
    where
        S: std::fmt::Display + std::fmt::Debug,
        D: std::fmt::Display + std::fmt::Debug,
    {
        debug!("{:?} = {:?}", dst, src);
        #[allow(clippy::to_string_in_format_args)]
        self.code.push_str(&format!("mov {} {}\n", dst, src));
    }

    pub fn clr<T>(&mut self, dst: Option<T>)
    where
        T: std::fmt::Display + std::fmt::Debug,
    {
        debug!("clr {:?}", dst.as_ref().map(|d| format!("{}", d)));

        let dst = match dst {
            Some(d) => format!(" {}\n", d),
            None => String::default(),
        };

        self.code.push_str(&format!("clr{}", dst));
    }

    // Move item to a CPU register
    pub fn reg_item(&mut self, reg: Register, val: &i32, sig: &Option<String>) {
        debug!("load immediate {}{:?} into {:?}", val, sig, reg);
        let item = format!("{}{}", val, sig.as_ref().unwrap_or(&String::new()));
        self.mov(reg, item);
    }

    // Move item to an outputting operand
    pub fn out_item(&mut self, out: Out, val: &i32, sig: &Option<String>) {
        debug!("write immediate {}{:?} to {:?}", val, sig, out);
        let item = format!("{}{}", val, sig.as_ref().unwrap_or(&String::new()));
        self.mov(out, item);
    }

    // Move register to an outputting operand
    pub fn out_reg(&mut self, out: Out, reg: Register) {
        debug!("write register {:?} to {:?}", reg, out);
        self.mov(out, reg);
    }

    // Arithmetic helper
    fn arith_op<O, D, S, V>(&mut self, op: &O, dst: D, src: Option<S>, val: V)
    where
        O: std::fmt::Display + std::fmt::Debug,
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        let rhs = match src {
            Some(s) => format!("{} {}", s, val),
            None => format!("{}", val),
        };

        let code = format!("{} {} {}\n", op, dst, rhs);
        self.code.push_str(&code);
    }

    // ADD
    pub fn add<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        match &src {
            Some(s) => debug!("{:?} = {:?} + {:?}", dst, s, val),
            None => debug!("{:?} = {:?} + {:?}", dst, dst, val),
        }
        self.arith_op(&"add", dst, src, val);
    }

    pub fn addi<D, V>(&mut self, dst: D, val: V)
    where
        D: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        self.add::<D, String, V>(dst, None, val);
    }

    // SUB
    pub fn sub<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        match &src {
            Some(s) => debug!("{:?} = {:?} - {:?}", dst, s, val),
            None => debug!("{:?} = {:?} - {:?}", dst, dst, val),
        }
        self.arith_op(&"sub", dst, src, val);
    }

    // MUL
    pub fn mul<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        match &src {
            Some(s) => debug!("{:?} = {:?} × {:?}", dst, s, val),
            None => debug!("{:?} = {:?} × {:?}", dst, dst, val),
        }
        self.arith_op(&"mul", dst, src, val);
    }

    pub fn muli<D, V>(&mut self, dst: D, val: V)
    where
        D: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        self.mul::<D, String, V>(dst, None, val);
    }

    // DIV
    pub fn div<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        match &src {
            Some(s) => debug!("{:?} = {:?} ÷ {:?}", dst, s, val),
            None => debug!("{:?} = {:?} ÷ {:?}", dst, dst, val),
        }
        self.arith_op(&"div", dst, src, val);
    }

    // MOD
    pub fn modu<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        match &src {
            Some(s) => debug!("{:?} = {:?} % {:?}", dst, s, val),
            None => debug!("{:?} = {:?} % {:?}", dst, dst, val),
        }
        self.arith_op(&"mod", dst, src, val);
    }
    pub fn inc<D>(&mut self, dst: D)
    where
        D: std::fmt::Display + std::fmt::Debug + Copy,
    {
        debug!("{:?} += 1", dst);
        self.code.push_str(&format!("inc {}\n", dst));
    }

    pub fn dec<D>(&mut self, dst: D)
    where
        D: std::fmt::Display + std::fmt::Debug,
    {
        debug!("{:?} -= 1", dst);
        self.code.push_str(&format!("inc {}\n", dst));
    }

    pub fn neg<D>(&mut self, dst: D)
    where
        D: std::fmt::Display + std::fmt::Debug,
    {
        debug!("-{:?}", dst);
        self.mul::<D, String, i32>(dst, None, -1);
    }

    pub fn not<D, S>(&mut self, dst: D, src: Option<S>)
    where
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
    {
        debug!("!{:?}", dst);
        self.code.push_str(&format!(
            "not {} {}\n",
            dst,
            match src {
                Some(s) => format!("{}", s),
                None => String::new(),
            }
        ));
    }

    pub fn finish(&self) -> &str {
        &self.code
    }
}

pub struct Assembler {
    instr: Vec<Instruction>,
    emitter: AssemblyEmitter,
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            instr: Vec::new(),
            emitter: AssemblyEmitter::new(),
        }
    }

    pub fn instructions(&self) -> &Vec<Instruction> {
        &self.instr
    }

    /// MOV: dst = src
    pub fn mov(&mut self, dst: Operand, src: Operand) {
        let instr = Instruction::Mov { dst, src };
        self.instr.push(instr);
    }

    /// ADD: dst = lhs + rhs
    pub fn add(&mut self, dst: Operand, lhs: Operand, rhs: Operand) {
        let instr = Instruction::BinOp {
            dst,
            lhs,
            rhs,
            op: BinOp::Add,
        };
        self.instr.push(instr);
    }

    /// SUB: dst = lhs - rhs
    pub fn sub(&mut self, dst: Operand, lhs: Operand, rhs: Operand) {
        let instr = Instruction::BinOp {
            dst,
            lhs,
            rhs,
            op: BinOp::Sub,
        };
        self.instr.push(instr);
    }

    /// MUL: dst = lhs * rhs
    pub fn mul(&mut self, dst: Operand, lhs: Operand, rhs: Operand) {
        let instr = Instruction::BinOp {
            dst,
            lhs,
            rhs,
            op: BinOp::Mul,
        };
        self.instr.push(instr);
    }

    /// DIV: dst = lhs / rhs
    pub fn div(&mut self, dst: Operand, lhs: Operand, rhs: Operand) {
        let instr = Instruction::BinOp {
            dst,
            lhs,
            rhs,
            op: BinOp::Div,
        };
        self.instr.push(instr);
    }

    /// MOD: dst = lhs % rhs
    pub fn modu(&mut self, dst: Operand, lhs: Operand, rhs: Operand) {
        let instr = Instruction::BinOp {
            dst,
            lhs,
            rhs,
            op: BinOp::Mod,
        };
        self.instr.push(instr);
    }

    /// Unary NOT: dst = !src
    pub fn not(&mut self, dst: Operand, src: Operand) {
        let instr = Instruction::UnaryOp {
            dst,
            src,
            op: UnaryOp::Not,
        };
        self.instr.push(instr);
    }

    /// Unary NEG: dst = -src
    pub fn neg(&mut self, dst: Operand, src: Operand) {
        let instr = Instruction::UnaryOp {
            dst,
            src,
            op: UnaryOp::Neg,
        };
        self.instr.push(instr);
    }

    /// Output instruction: send src to signal
    pub fn out(&mut self, src: Operand, signal_id: Option<String>) {
        let instr = Instruction::Out { src, signal_id };
        self.instr.push(instr);
    }

    /// Move signal
    pub fn mov_sig(&mut self, dst: Operand, src: Operand, signal_id: Option<String>) {
        let instr = Instruction::MovSig {
            dst,
            src,
            signal_id,
        };
        self.instr.push(instr);
    }

    /// Push a NOP
    pub fn nop(&mut self) {
        self.instr.push(Instruction::Nop);
    }
    pub fn finish(&mut self) -> &str {
        debug!("Emittation ended ({} bytes)", self.emitter.code.len());
        self.emitter.finish()
    }
}
