use super::emit::*;
use super::ir::*;
use super::low::*;
use super::mem::*;
use super::symbol::*;
use super::tags::*;
use crate::error::*;

use crate::frontend::ast::UnaryOp;
use crate::log;

#[derive(Default)]
pub struct Assembler {
    code: String,
    memory: MemoryManager,
    outputs: OutputManager,
    tape: Tape,
    scopes: ScopeArena,
}

impl Assembler {
    pub fn new(lowered: Lowerer) -> Self {
        Self {
            tape: lowered.tape().clone(),
            scopes: lowered.scopes().clone(),
            memory: lowered.memory().clone(),
            ..Default::default()
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn assemble(&mut self) -> Result<(), CompileError> {
        let instructions = self.tape.instrs.clone();
        let mut tmps = self.tape.count_temp();

        for (_, instr) in instructions.iter().enumerate() {
            log::asm!("{:?}", instr);
            let asm_line = match instr {
                Instruction::BinOp { dst, lhs, rhs, op } => {
                    let dst = self.ensure_location(dst)?;
                    let lhs = self.ensure_location(lhs)?;
                    let rhs = self.ensure_location(rhs)?;

                    match (dst, lhs, rhs) {
                        (Resolved::Reg(dst), Resolved::Reg(lhs), Resolved::Reg(rhs)) => {
                            AsmFormatter::arith(&op.to_string(), dst, lhs, rhs)
                        }
                        (Resolved::Reg(dst), Resolved::Reg(r), Resolved::Imm(n)) => {
                            AsmFormatter::arith(&op.to_string(), dst, r, n)
                        }
                        (Resolved::Reg(dst), Resolved::Imm(n), Resolved::Reg(r)) => {
                            if op.is_commutative() {
                                AsmFormatter::arith(&op.to_string(), dst, r, n)
                            } else {
                                AsmFormatter::arith(&op.to_string(), dst, n, r)
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                Instruction::UnaryOp { dst, src, op } => {
                    let dst = self.ensure_location(dst)?;
                    let src = self.ensure_location(src)?;

                    let lhs = if dst == src { "-1" } else { &src.to_string() };
                    let rhs = if dst == src { "" } else { "-1" };

                    match op {
                        UnaryOp::Not => AsmFormatter::not(dst, Some(src)),
                        UnaryOp::Neg => AsmFormatter::arith("mul", dst, lhs, rhs),
                    }
                }
                Instruction::Mov { dst, src } => {
                    let dst = self.ensure_location(dst)?;
                    let src = self.ensure_location(src)?;
                    AsmFormatter::mov(dst, src)
                }
                Instruction::MovSig {
                    dst,
                    src,
                    signal_id,
                } => {
                    let dst = self.ensure_location(dst)?;
                    let src = self.ensure_location(src)?;

                    if !src.is_imm() {
                        return Err(CompileError::new(
                            CompileErrorKind::Generation(GeneratorError::RegisterCannotBeTyped),
                            None,
                        ));
                    }

                    AsmFormatter::mov(dst, format!("{}{}", src, signal_id.format()))
                }
                Instruction::Out { src, signal_id } => {
                    let src = self.ensure_location(src)?;
                    let item = format!(
                        "{}{}",
                        src,
                        signal_id.map(|s| s.format()).unwrap_or_default()
                    );
                    AsmFormatter::mov(self.outputs.out(), item)
                }
                Instruction::Nop => String::new(),
                Instruction::Inc { dst } => {
                    let dst = self.ensure_location(dst)?;
                    AsmFormatter::inc(dst)
                }
                Instruction::Dec { dst } => {
                    let dst = self.ensure_location(dst)?;
                    AsmFormatter::dec(dst)
                }
                Instruction::Compare { a, b, op, addr } => {
                    let lhs = self.ensure_location(a)?;
                    let rhs = self.ensure_location(b)?;

                    if let Some(addr) = addr {
                        AsmFormatter::branch(&op.branch_op(), lhs, rhs, addr)
                    } else {
                        AsmFormatter::test(&op.test_op(), lhs, rhs)
                    }
                }
                Instruction::Label { name } => AsmFormatter::label(name.to_string()),
                Instruction::Jump { addr, offset } => AsmFormatter::jmp(
                    addr,
                    if let Some(offset) = offset {
                        offset.to_string()
                    } else {
                        "".to_string()
                    },
                ),
                Instruction::Event(context) => {
                    match context {
                        EventContext::ScopeEntered { scope_idx } => {
                            if let Some(scope) = self.scopes.get(*scope_idx) {
                                log::warn!(" {:?}:{}", scope.metadata.kind, scope.metadata.idx());
                            }
                        }
                        EventContext::ScopeDropped { scope_idx } => {
                            if let Some(scope) = self.scopes.drop_scope(scope_idx) {
                                if scope.metadata.kind.is_global() {
                                    continue;
                                }

                                log::warn!(" {:?}:{}", scope.metadata.kind, scope.metadata.idx());
                                for sym in scope.locals.values() {
                                    match sym.loc {
                                        Location::Reg(reg) => {
                                            self.memory.free(reg);
                                        }
                                        Location::Stk(_) => todo!(),
                                    }
                                }
                            }
                        }
                        EventContext::Free { oprs } => {
                            let mut regs = Vec::new();
                            for opr in oprs {
                                regs.push(self.ensure_location(opr)?);
                            }

                            let mut clear_regs = String::new();
                            for r in regs {
                                clear_regs.push_str(&format!("{} ", r));
                            }

                            self.code.push_str(&AsmFormatter::clr(Some(clear_regs)));
                        }
                    }

                    String::new()
                }
            };

            if !asm_line.is_empty() {
                self.code.push_str(&asm_line);
            }

            for src in instr.sources() {
                if let Operand::Temp(temp_id) = src
                    && let Some(count) = tmps.get_mut(temp_id)
                {
                    *count -= 1;
                    if *count == 0
                        && let Some(reg) = self.memory.temps.remove(temp_id)
                    {
                        self.memory.free(reg);
                        log::warn!("Freed {:?} ← {:?}", reg, temp_id);
                    }
                }
            }
        }

        let dead_marks = self.memory.dead_marks();
        if !dead_marks.is_empty() {
            let mut clear_regs = String::new();
            for dead_reg in dead_marks {
                clear_regs.push_str(&format!("{} ", dead_reg));
            }

            self.code.push_str(&AsmFormatter::clr(Some(clear_regs)));
        }

        Ok(())
    }

    fn ensure_location(&mut self, opr: &Operand) -> Result<Resolved, CompileError> {
        match opr {
            Operand::Persistent(sid) => match self.scopes.snatch(sid) {
                Some(sym) => Ok(Resolved::Reg(sym.loc.into())),
                None => Err(CompileError::new(
                    CompileErrorKind::Generation(GeneratorError::NonAddressableSymbol {
                        ctx: sid.to_string(),
                    }),
                    None,
                )),
            },
            Operand::Temp(temp_id) => {
                let reg = match self.memory.temps.get(temp_id) {
                    Some(r) => *r,
                    None => {
                        let reg = self
                            .memory
                            .alloc()
                            .map_err(|k| CompileError::new(k, None))?;
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
}
