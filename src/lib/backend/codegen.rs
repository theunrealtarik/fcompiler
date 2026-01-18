use std::ops::Deref;

use super::asm::*;
use super::mem::*;
use super::symbol::*;

use crate::backend::ir::Operand;
use crate::error::*;
use crate::frontend::ast::*;

#[derive(Default)]
#[allow(dead_code)]
pub struct Generator {
    program: Program,
    symbols: SymbolTable,
    asm: Assembler,
    memory: MemoryManager,
    outputs: OutputManager,
}

impl Generator {
    pub fn new(program: Program) -> Self {
        Self {
            program,
            symbols: SymbolTable::new(),
            asm: Assembler::new(),
            memory: MemoryManager::new(),
            outputs: OutputManager::default(),
        }
    }

    pub fn generate(&mut self) -> Result<&str, CompileError> {
        match self.handle_statements() {
            Ok(_) => {
                for instr in self.asm.instructions() {
                    log::debug!("{:?}", instr);
                }

                Ok("")
            }
            Err((kind, span)) => Err(CompileError::new(kind, Some(span))),
        }
    }

    fn handle_statements(&mut self) -> Result<(), (CompileErrorKind, Span)> {
        for stmt in self.program.clone() {
            match stmt.kind {
                StatementKind::Declare { ident, sigid, expr } => {
                    let reg = self.memory.alloc().map_err(|k| (k, stmt.span))?;
                    let var = Symbol::new(ident, Location::REG(reg), sigid);

                    let dst = Operand::persistent();
                    let opr = self
                        .proc_expr(&expr, Some(dst))
                        .map_err(|k| (k, stmt.span))?;

                    self.symbols.push(&SymbolId(dst.into()), var);

                    if dst != opr {
                        self.asm.mov(dst, opr);
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

                    let dst = Operand::Persistent(**sid);
                    let opr = self.proc_expr(&expr, None).map_err(|k| (k, stmt.span))?;

                    self.asm.mov(dst, opr);
                }
                StatementKind::Out(signal) => match signal.value {
                    SignalValue::Num(scalar) => {
                        let signal = signal.id.map(|s| s.format());
                        self.asm.out(Operand::Imm(scalar), signal);
                    }
                    SignalValue::Var(ident) => {
                        if let Some((sid, _)) = self.symbols.lookup(&ident) {
                            let target = Operand::Persistent(**sid);
                            let signal_id = signal.id.map(|s| s.format());

                            if signal.id.is_some() {
                                let signal_id = signal_id.clone();
                                let caster = Operand::temp();

                                self.asm.mov_sig(caster, Operand::Imm(1), signal_id);
                                self.asm.mul(caster, target, target);
                                self.asm.mov(target, caster);
                            }

                            self.asm.out(target, signal_id);
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
                    Some((sid, _)) => Ok(Operand::Persistent(**sid)),
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

                    // self.asm.mul(dst, opr, Operand::Imm(-1));
                    self.asm.neg(dst, opr);
                    Ok(dst)
                }
                UnaryOp::Not => {
                    let dst = p_dst.unwrap_or(Operand::temp());
                    let src = self.proc_expr(expr, Some(dst))?;
                    self.asm.not(dst, src);
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
            BinOp::Add => self.asm.add(dst, src, val),
            BinOp::Sub => self.asm.sub(dst, src, val),
            BinOp::Mul => self.asm.mul(dst, src, val),
            BinOp::Div => self.asm.div(dst, src, val),
            BinOp::Mod => self.asm.modu(dst, src, val),
        }
    }
}
