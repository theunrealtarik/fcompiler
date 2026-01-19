mod emit;
mod ir;

pub use emit::*;
pub use ir::*;

use std::collections::HashMap;
use std::ops::Deref;

use super::mem::*;
use super::symbol::*;

use crate::error::*;
use crate::frontend::ast::*;

#[derive(Default)]
pub struct Assembler {
    instr: Vec<Instruction>,
    code: String,
    program: Program,
    symbols: SymbolTable,
    memory: MemoryManager,
    outputs: OutputManager,
}

impl Assembler {
    pub fn new(program: Program) -> Self {
        Self {
            program,
            ..Default::default()
        }
    }

    pub fn instructions(&self) -> &Vec<Instruction> {
        &self.instr
    }

    pub fn finish(&mut self) -> Result<&str, CompileError> {
        self.handle_statements()
            .map_err(|(k, span)| CompileError::new(k, Some(span)))?;

        let instr = self.instr.clone();
        let mut tmps = Self::count_temp(&instr);

        self.code.push_str(&AssemblyFormatter::clr::<String>(None));
        for instr in instr.iter() {
            match instr {
                Instruction::BinOp { dst, lhs, rhs, op } => {
                    let dst = self.ensure_location(dst).unwrap();
                    let lhs = self.ensure_location(lhs).unwrap();
                    let rhs = self.ensure_location(rhs).unwrap();

                    match (dst, lhs, rhs) {
                        (Resolved::Reg(dst), Resolved::Reg(lhs), Resolved::Reg(rhs)) => self
                            .code
                            .push_str(&AssemblyFormatter::arith(&op.to_string(), dst, lhs, rhs)),

                        (Resolved::Reg(dst), Resolved::Reg(r), Resolved::Imm(n)) => self
                            .code
                            .push_str(&AssemblyFormatter::arith(&op.to_string(), dst, r, n)),

                        (Resolved::Reg(dst), Resolved::Imm(n), Resolved::Reg(r)) => {
                            if op.is_commutative() {
                                self.code.push_str(&AssemblyFormatter::arith(
                                    &op.to_string(),
                                    dst,
                                    r,
                                    n,
                                ))
                            } else {
                                self.code.push_str(&AssemblyFormatter::arith(
                                    &op.to_string(),
                                    dst,
                                    n,
                                    r,
                                ))
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                Instruction::UnaryOp { dst, src, op } => {
                    let dst = self.ensure_location(dst).unwrap();
                    let src = self.ensure_location(src).unwrap();

                    match op {
                        UnaryOp::Not => self.code.push_str(&AssemblyFormatter::not(dst, Some(src))),
                        UnaryOp::Neg => self.code.push_str(&AssemblyFormatter::neg(dst)),
                    }
                }
                Instruction::Mov { dst, src } => {
                    let dst = self.ensure_location(dst).unwrap();
                    let src = self.ensure_location(src).unwrap();
                    self.code.push_str(&AssemblyFormatter::mov(dst, src))
                }
                Instruction::MovSig {
                    dst,
                    src,
                    signal_id,
                } => {
                    let dst = self.ensure_location(dst).unwrap();
                    let src = self.ensure_location(src).unwrap();

                    if !src.is_imm() {
                        return Err(CompileError::new(
                            CompileErrorKind::Generation(GeneratorError::RegisterCannotBeTyped),
                            None,
                        ));
                    }

                    self.code.push_str(&AssemblyFormatter::mov(
                        dst,
                        format!("{}{}", src, signal_id.format()),
                    ))
                }
                Instruction::Out { src, signal_id } => {
                    let src = self.ensure_location(src).unwrap();
                    let item = format!(
                        "{}{}",
                        src,
                        signal_id.map(|s| s.format()).unwrap_or(String::new())
                    );

                    self.code
                        .push_str(&AssemblyFormatter::mov(self.outputs.out(), item))
                }
                Instruction::Nop => {}
            }

            log::debug!("{:?}", instr);
            for src in instr.sources() {
                if let Operand::Temp(temp_id) = src
                    && let Some(count) = tmps.get_mut(temp_id)
                {
                    *count -= 1;
                    if *count == 0
                        && let Some(reg) = self.memory.temps.remove(temp_id)
                    {
                        self.memory.free(reg);
                        log::warn!("Freed {:?} ) → {:?}", reg, temp_id);
                    }
                }
            }
        }

        let mut clear_regs = String::new();
        for dead_reg in self.memory.dead_marks() {
            clear_regs.push_str(&format!("{} ", dead_reg));
        }

        self.code
            .push_str(&AssemblyFormatter::clr(Some(clear_regs)));

        Ok(&self.code)
    }

    fn ensure_location(&mut self, opr: &Operand) -> Result<Resolved, CompileErrorKind> {
        match opr {
            Operand::Persistent(symbol_id) => match self.symbols.get(symbol_id) {
                Some(symbol) => Ok(Resolved::Reg(symbol.loc.into())),
                None => Err(CompileErrorKind::Generation(
                    GeneratorError::NonAddressableSymbol,
                )),
            },
            Operand::Temp(temp_id) => {
                let reg = match self.memory.temps.get(temp_id) {
                    Some(r) => *r,
                    None => {
                        let reg = self.memory.alloc()?;
                        self.memory.temps.insert(*temp_id, reg);
                        log::warn!("Allocated {:?} → {:?}", reg, temp_id);
                        reg
                    }
                };

                Ok(Resolved::Reg(reg))
            }
            Operand::Imm(n) => Ok(Resolved::Imm(*n)),
        }
    }

    fn count_temp(instr: &[Instruction]) -> HashMap<&TempId, u32> {
        let mut map: HashMap<&TempId, u32> = HashMap::new();
        for intr in instr.iter() {
            for opr in intr.sources() {
                if let Operand::Temp(id) = opr {
                    map.entry(id).and_modify(|count| *count += 1).or_insert(1);
                }
            }
        }

        map
    }

    fn handle_statements(&mut self) -> Result<(), (CompileErrorKind, Span)> {
        for stmt in self.program.clone() {
            match stmt.kind {
                StatementKind::Declare { ident, sigid, expr } => {
                    let reg = self.memory.alloc().map_err(|k| (k, stmt.span))?;
                    let var = Symbol::new(ident, Location::Reg(reg), sigid);

                    let dst = Operand::persistent();
                    let opr = self
                        .proc_expr(&expr, Some(dst))
                        .map_err(|k| (k, stmt.span))?;

                    self.symbols.push(&SymbolId(dst.into()), var);

                    if dst != opr {
                        self.mov(dst, opr);
                    }
                }
                StatementKind::Assign { ident, expr } => {
                    let (sid, _) = match self.symbols.lookup(&ident) {
                        Some((sid, symb)) => (sid, symb),
                        None => {
                            return Err((
                                CompileErrorKind::Semantic(SemanticError::UndefinedVariable(
                                    ident.to_string(),
                                )),
                                stmt.span,
                            ));
                        }
                    };

                    let dst = Operand::Persistent(*sid);
                    let opr = self.proc_expr(&expr, None).map_err(|k| (k, stmt.span))?;

                    self.mov(dst, opr);
                }
                StatementKind::Out(signal) => match signal.value {
                    SignalValue::Num(scalar) => {
                        self.out(Operand::Imm(scalar), signal.id);
                    }
                    SignalValue::Var(ident) => {
                        if let Some((sid, _)) = self.symbols.lookup(&ident) {
                            let target = Operand::Persistent(*sid);

                            if let Some(signal_id) = signal.id {
                                let caster = Operand::temp();

                                self.mov_sig(caster, Operand::Imm(1), signal_id);
                                self.mul(caster, target, target);
                                self.mov(target, caster);
                            }

                            self.out(target, signal.id);
                        } else {
                            return Err((
                                CompileErrorKind::Semantic(SemanticError::UndefinedVariable(
                                    ident.clone(),
                                )),
                                stmt.span,
                            ));
                        }
                    }
                },
            }
        }

        Ok(())
    }

    fn proc_expr(
        &mut self,
        expr: &Expression,
        p_dst: Option<Operand>,
    ) -> Result<Operand, CompileErrorKind> {
        match expr {
            Expression::Value(signal) => match &signal.value {
                SignalValue::Num(n) => Ok(Operand::immediate(*n)),
                SignalValue::Var(r_ident) => match self.symbols.lookup(r_ident) {
                    Some((sid, _)) => Ok(Operand::Persistent(*sid)),
                    None => Err(CompileErrorKind::Semantic(
                        SemanticError::UndefinedVariable(r_ident.to_string()),
                    )),
                },
            },
            Expression::Op { lhs, rhs, op } => {
                let lhs = (lhs.deref()).clone();
                let rhs = (rhs.deref()).clone();

                let lhs_opr = self.proc_expr(&lhs, None)?;
                let rhs_opr = self.proc_expr(&rhs, None)?;

                let dst = p_dst.unwrap_or_else(Operand::temp);
                self.lower_op(lhs_opr, rhs_opr, op, dst)
            }
            Expression::UnaryOp { expr, op } => match op {
                UnaryOp::Neg => {
                    let dst = p_dst.unwrap_or(Operand::temp());
                    let opr = self.proc_expr(expr, Some(dst))?;

                    // self.mul(dst, opr, Operand::Imm(-1));
                    self.neg(dst, opr);
                    Ok(dst)
                }
                UnaryOp::Not => {
                    let dst = p_dst.unwrap_or(Operand::temp());
                    let src = self.proc_expr(expr, Some(dst))?;
                    self.not(dst, src);
                    Ok(dst)
                }
            },
        }
    }

    fn lower_op(
        &mut self,
        lhs: Operand,
        rhs: Operand,
        op: &BinOp,
        dst: Operand,
    ) -> Result<Operand, CompileErrorKind> {
        match (lhs, rhs) {
            // X: N OP M
            (Operand::Imm(n), Operand::Imm(m)) => {
                let r = match op {
                    BinOp::Add => n + m,
                    BinOp::Sub => n - m,
                    BinOp::Mul => n * m,
                    BinOp::Div => n / m,
                    BinOp::Mod => n % m,
                };

                Ok(Operand::Imm(r))
            }

            // X: N OP R
            (Operand::Imm(n), Operand::Persistent(_) | Operand::Temp(_)) => {
                if op.is_commutative() {
                    return self.lower_op(rhs, Operand::Imm(n), op, dst);
                }
                self.proc_op(op, dst, Some(lhs), rhs);
                Ok(dst)
            }

            // X: R OP R
            // X: R OP N
            _ => {
                self.proc_op(op, dst, Some(lhs), rhs);
                Ok(dst)
            }
        }
    }

    fn proc_op(&mut self, op: &BinOp, dst: Operand, src: Option<Operand>, val: Operand) {
        let src = match src {
            Some(opr) => opr,
            None => dst,
        };

        match op {
            BinOp::Add => self.add(dst, src, val),
            BinOp::Sub => self.sub(dst, src, val),
            BinOp::Mul => self.mul(dst, src, val),
            BinOp::Div => self.div(dst, src, val),
            BinOp::Mod => self.modu(dst, src, val),
        }
    }
}
/// IR emitting helpers
impl Assembler {
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
    pub fn out(&mut self, src: Operand, signal_id: Option<crate::game::SignalId>) {
        let instr = Instruction::Out { src, signal_id };
        self.instr.push(instr);
    }

    /// Move signal
    pub fn mov_sig(&mut self, dst: Operand, src: Operand, signal_id: crate::game::SignalId) {
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
}
