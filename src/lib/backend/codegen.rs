use std::ops::Deref;

use super::asm::*;
use super::mem::*;
use super::symbol::*;

use crate::error::CompileError;
use crate::frontend::ast::*;

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Generator {
    symbols: SymbolTable,
    asm: Asm,
    program: Program,
    registors: RegisterAllocator,
    stack: StackAllocator,
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
            match stmt {
                Statement::Let { ident, sigid, expr } => {
                    let reg = match self.proc_expr(&expr, &sigid).unwrap() {
                        Location::REG(r) => r,
                        Location::IMM(n) => {
                            let r = self.registors.alloc().unwrap();
                            self.asm.mov(r, n);
                            r
                        }
                        Location::STK(_) => todo!(),
                    };

                    self.symbols.insert(
                        ident.to_string(),
                        Variable::new(ident.to_string(), Location::REG(reg), sigid),
                    );
                }
                Statement::Out(signal) => match signal.value {
                    SignalValue::Num(scalar) => {
                        let signal = signal.id.map(|s| s.format());
                        self.asm.out_item(self.outputs.out(), &scalar, &signal);
                    }
                    SignalValue::Var(ident) => {
                        if let Some(var) = self.symbols.get_mut(&ident) {
                            if let Some(sigid) = signal.id {
                                let caster = self.registors.alloc().unwrap();
                                self.asm.reg_item(caster, &1, &Some(sigid.format()));
                                self.asm.mul::<_, String, _>(caster, None, var.loc);

                                // self.registors.free(var.reg);
                                self.asm.clr(Some(var.loc));
                                var.loc = Location::REG(caster);
                            }

                            match var.loc {
                                Location::REG(r) => self.asm.out_reg(self.outputs.out(), r),
                                _ => continue,
                            }
                        } else {
                            return Err(CompileError::UndefinedVariable(ident.clone()));
                        }
                    }
                },
            }
        }

        dbg!(&self.symbols);
        Ok(self.asm.finish())
    }

    fn proc_expr(
        &mut self,
        expr: &Expression,
        parent_sigid: &Option<crate::game::SignalId>,
    ) -> Result<Location, CompileError> {
        match expr {
            Expression::Value(signal) => match &signal.value {
                SignalValue::Num(n) => Ok(Location::IMM(*n)),
                SignalValue::Var(r_ident) => {
                    let var = self
                        .symbols
                        .get(r_ident)
                        .ok_or_else(|| CompileError::UndefinedVariable(r_ident.clone()))
                        .unwrap();

                    match var.loc {
                        Location::REG(r) => Ok(Location::REG(r)),
                        _ => todo!(),
                    }
                }
            },
            Expression::Op { lhs, rhs, op } => {
                let lhs = (lhs.deref()).clone();
                let rhs = (rhs.deref()).clone();

                let lhs_loc = self.proc_expr(&lhs, parent_sigid).unwrap();
                let rhs_loc = self.proc_expr(&rhs, parent_sigid).unwrap();

                self.lower_op(lhs_loc, rhs_loc, op)
            }
        }
    }

    fn lower_op(
        &mut self,
        lhs: Location,
        rhs: Location,
        op: &Sign,
    ) -> Result<Location, CompileError> {
        match (lhs, rhs) {
            // X: R OP R
            (Location::REG(lhs), Location::REG(rhs)) => {
                let dst = self.registors.alloc().unwrap();

                match op {
                    Sign::Add => self.asm.add(dst, Some(lhs), rhs),
                    Sign::Sub => self.asm.sub(dst, Some(lhs), rhs),
                    Sign::Mul => self.asm.mul(dst, Some(lhs), rhs),
                    Sign::Div => self.asm.div(dst, Some(lhs), rhs),
                    Sign::Mod => self.asm.modu(dst, Some(lhs), rhs),
                }

                Ok(Location::REG(dst))
            }

            // X: N OP M
            (Location::IMM(n), Location::IMM(m)) => {
                let r = match op {
                    Sign::Add => n + m,
                    Sign::Sub => n - m,
                    Sign::Mul => n * m,
                    Sign::Div => n / m,
                    Sign::Mod => n % m,
                };

                let reg = self.registors.alloc().unwrap();
                self.asm.mov(reg, r);
                Ok(Location::REG(reg))
            }

            (Location::REG(lhs), Location::IMM(n)) => {
                let lhs_is_symbol = self.symbols.values().any(|v| match v.loc {
                    Location::REG(r) => r == lhs,
                    _ => false,
                });

                dbg!(&lhs_is_symbol);

                let dst = self.registors.alloc().unwrap();
                self.asm.mov(dst, lhs);
                match op {
                    Sign::Add => self.asm.addi(dst, n),
                    Sign::Sub => self.asm.subi(dst, n),
                    Sign::Mul => self.asm.mul(dst, None::<Register>, n),
                    Sign::Div => self.asm.div(dst, None::<Register>, n),
                    Sign::Mod => self.asm.modu(dst, None::<Register>, n),
                }

                if !lhs_is_symbol {
                    return Ok(Location::REG(lhs));
                }

                Ok(Location::REG(dst))
            }

            // N OP R -> make immediate into reg then three-operand
            (Location::IMM(n), Location::REG(rhs)) => {
                let lhs = self.registors.alloc().unwrap();
                self.asm.reg_item(lhs, &n, &None);
                let dst = self.registors.alloc().unwrap();
                match op {
                    Sign::Add => self.asm.add(dst, Some(lhs), rhs),
                    Sign::Sub => self.asm.sub(dst, Some(lhs), rhs),
                    Sign::Mul => self.asm.mul(dst, Some(lhs), rhs),
                    Sign::Div => self.asm.div(dst, Some(lhs), rhs),
                    Sign::Mod => self.asm.modu(dst, Some(lhs), rhs),
                }
                Ok(Location::REG(dst))
            }
            _ => unimplemented!("CASE NOT IMPLEMENTED"),
        }
    }
}
