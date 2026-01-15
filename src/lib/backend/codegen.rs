use std::ops::Deref;

use super::asm::*;
use super::mem::*;
use super::symbol::*;

use crate::error::*;
use crate::frontend::ast::*;

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Generator {
    symbols: SymbolTable,
    asm: Asm,
    program: Program,
    memory: MemoryManager,
    outputs: OutputManager,
}

impl Generator {
    pub fn new(program: Program) -> Self {
        Self {
            program,
            ..Default::default()
        }
    }

    pub fn generate(&mut self) -> Result<&str, CompileError> {
        for stmt in self.program.clone() {
            match stmt.kind {
                StatementKind::Declare { ident, sigid, expr } => match self.proc_expr(&expr) {
                    Ok(loc) => {
                        let reg = self.ensure_reg(loc);
                        self.symbols.insert(
                            ident.to_string(),
                            Variable::new(ident.to_string(), VariableLocation::REG(reg), sigid),
                        );
                    }
                    Err(kind) => return Err(CompileError::new(kind, Some(stmt.span))),
                },
                StatementKind::Assign { ident, expr } => {
                    let var = match self.symbols.get(&ident) {
                        Some(v) => v,
                        None => {
                            return Err::<&str, CompileError>(CompileError::new(
                                CompileErrorKind::Semantic(SemanticError::UndefinedVariable(ident)),
                                Some(stmt.span),
                            ));
                        }
                    };

                    let lhs_reg = *var.loc.as_register();
                    let rhs_loc = match self.proc_expr(&expr) {
                        Ok(location) => location,
                        Err(kind) => return Err(CompileError::new(kind, Some(stmt.span))),
                    };

                    let rhs_reg = self.ensure_reg(rhs_loc);
                    if rhs_reg != lhs_reg {
                        self.asm.mov(lhs_reg, rhs_reg);
                        self.memory.free(rhs_reg);
                    }
                }
                StatementKind::Out(signal) => match signal.value {
                    SignalValue::Num(scalar) => {
                        let signal = signal.id.map(|s| s.format());
                        self.asm.out_item(self.outputs.out(), &scalar, &signal);
                    }
                    SignalValue::Var(ident) => {
                        if let Some(var) = self.symbols.get_mut(&ident) {
                            if let Some(sigid) = signal.id {
                                let caster = self.memory.alloc().unwrap();
                                let reg = match var.loc {
                                    VariableLocation::REG(r) => r,
                                    VariableLocation::STK(_) => todo!(),
                                };

                                self.asm.reg_item(caster, &1, &Some(sigid.format()));
                                self.asm.mul::<_, String, _>(caster, None, reg);

                                self.memory.free(reg);
                                var.loc = VariableLocation::REG(caster);
                            }
                            self.asm.out_reg(self.outputs.out(), var.loc.into());
                        } else {
                            return Err(CompileError::new(
                                CompileErrorKind::Semantic(SemanticError::UndefinedVariable(
                                    ident.clone(),
                                )),
                                Some(stmt.span),
                            ));
                        }
                    }
                },
            }
        }

        log::debug!("{:#?}", self.memory.dead_marks());
        Ok(self.asm.finish())
    }

    fn proc_expr(&mut self, expr: &Expression) -> Result<OperandLocation, CompileErrorKind> {
        match expr {
            Expression::Value(signal) => match &signal.value {
                SignalValue::Num(n) => Ok(OperandLocation::IMM(*n)),
                SignalValue::Var(r_ident) => {
                    let var = self
                        .symbols
                        .get(r_ident)
                        .ok_or_else(|| {
                            CompileErrorKind::Semantic(SemanticError::UndefinedVariable(
                                r_ident.clone(),
                            ))
                        })
                        .unwrap();

                    Ok(var.loc.into())
                }
            },
            Expression::Op { lhs, rhs, op } => {
                let lhs = (lhs.deref()).clone();
                let rhs = (rhs.deref()).clone();

                let lhs_loc = self.proc_expr(&lhs)?;
                let rhs_loc = self.proc_expr(&rhs)?;

                self.lower_op(lhs_loc, rhs_loc, op)
            }
            Expression::UnaryOp { expr, op } => {
                let loc = match self.proc_expr(expr)? {
                    OperandLocation::IMM(n) => OperandLocation::IMM(-n),
                    opr => opr,
                };

                let mut rhs = self.ensure_reg(loc);
                match op {
                    UnarySign::Neg => {
                        if !loc.is_imm() {
                            let dst = self.memory.alloc().unwrap();
                            if rhs == dst {
                                self.asm.muli(dst, -1);
                            } else {
                                self.asm.mul(dst, Some(rhs), -1);
                            }

                            self.free_unmapped(rhs);
                            rhs = dst;
                        }
                    }
                    UnarySign::Not => self.asm.not(rhs),
                }

                Ok(OperandLocation::REG(rhs))
            }
        }
    }

    fn lower_op(
        &mut self,
        lhs: OperandLocation,
        rhs: OperandLocation,
        op: &Sign,
    ) -> Result<OperandLocation, CompileErrorKind> {
        match (lhs, rhs) {
            // X: R OP R
            (OperandLocation::REG(lhs), OperandLocation::REG(rhs)) => {
                let dst = self.memory.alloc().unwrap();
                self.omit_op(op, dst, Some(lhs), rhs);

                self.free_unmapped(lhs);
                self.free_unmapped(rhs);

                Ok(OperandLocation::REG(dst))
            }

            // X: N OP M
            (OperandLocation::IMM(n), OperandLocation::IMM(m)) => {
                let r = match op {
                    Sign::Add => n + m,
                    Sign::Sub => n - m,
                    Sign::Mul => n * m,
                    Sign::Div => n / m,
                    Sign::Mod => n % m,
                };

                Ok(OperandLocation::IMM(r))
            }

            // X: R OP N
            (OperandLocation::REG(lhs), OperandLocation::IMM(n)) => {
                let dst = self.memory.alloc().unwrap();
                self.omit_op(op, dst, Some(lhs), n);
                self.free_unmapped(lhs);
                self.free_unmapped(dst);

                Ok(OperandLocation::REG(dst))
            }

            // X: N OP R
            (OperandLocation::IMM(n), OperandLocation::REG(rhs)) => {
                if op.is_commutative() {
                    return self.lower_op(OperandLocation::REG(rhs), OperandLocation::IMM(n), op);
                }

                let dst = self.memory.alloc().unwrap();
                self.omit_op(op, dst, Some(rhs), n);
                self.free_unmapped(rhs);

                Ok(OperandLocation::REG(dst))
            }
            _ => unimplemented!("CASE NOT IMPLEMENTED"),
        }
    }

    fn omit_op<D, S, V>(&mut self, op: &Sign, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        match op {
            Sign::Add => self.asm.add(dst, src, val),
            Sign::Sub => self.asm.sub(dst, src, val),
            Sign::Mul => self.asm.mul(dst, src, val),
            Sign::Div => self.asm.div(dst, src, val),
            Sign::Mod => self.asm.modu(dst, src, val),
        }
    }

    fn free_unmapped(&mut self, r: Register) {
        let sym = self.symbols.get_by_register(&r);
        if sym.is_none() {
            self.memory.free(r);
        }
    }

    fn ensure_reg(&mut self, loc: OperandLocation) -> Register {
        match loc {
            OperandLocation::REG(r) => r,
            OperandLocation::STK(_s) => todo!(),
            OperandLocation::IMM(n) => {
                let r = self.memory.alloc().unwrap();
                self.asm.mov(r, n);
                r
            }
        }
    }
}
