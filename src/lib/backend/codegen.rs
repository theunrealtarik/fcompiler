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
                            match sigid {
                                Some(s) => self.asm.reg_item(r, &n, &Some(s.format())),
                                None => self.asm.mov(r, n),
                            }

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
                self.omit_op(op, dst, Some(lhs), rhs);

                self.free_unmapped(lhs);
                self.free_unmapped(rhs);

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

                // let reg = self.registors.alloc().unwrap();
                // self.asm.mov(reg, r);
                Ok(Location::IMM(r))
            }

            // X: R OP N
            (Location::REG(lhs), Location::IMM(n)) => {
                let dst = self.registors.alloc().unwrap();
                self.omit_op(op, dst, Some(lhs), n);
                self.free_unmapped(lhs);
                Ok(Location::REG(dst))
            }

            // X: N OP R
            (Location::IMM(n), Location::REG(rhs)) => {
                if op.is_commutative() {
                    return self.lower_op(Location::REG(rhs), Location::IMM(n), op);
                }

                let dst = self.registors.alloc().unwrap();
                self.omit_op(op, dst, Some(rhs), n);
                self.free_unmapped(rhs);

                Ok(Location::REG(dst))
            }
            _ => unimplemented!("CASE NOT IMPLEMENTED"),
        }
    }

    fn omit_op<D, S, V>(&mut self, op: &Sign, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display,
        S: std::fmt::Display,
        V: std::fmt::Display,
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
            self.registors.free(r);
        }
    }
}
