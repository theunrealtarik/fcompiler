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
            match stmt.kind {
                StatementKind::Let { ident, sigid, expr } => match self.proc_expr(&expr) {
                    Ok(loc) => {
                        let reg = match loc {
                            Location::REG(r) => r,
                            Location::STK(_s) => todo!(),
                            Location::IMM(n) => {
                                let r = self.registors.alloc().unwrap();
                                match sigid {
                                    Some(s) => self.asm.reg_item(r, &n, &Some(s.format())),
                                    None => self.asm.mov(r, n),
                                }

                                r
                            }
                        };

                        self.symbols.insert(
                            ident.to_string(),
                            Variable::new(ident.to_string(), Location::REG(reg), sigid),
                        );
                    }
                    Err(kind) => return Err(CompileError::new(kind, Some(stmt.span))),
                },
                StatementKind::Out(signal) => match signal.value {
                    SignalValue::Num(scalar) => {
                        let signal = signal.id.map(|s| s.format());
                        self.asm.out_item(self.outputs.out(), &scalar, &signal);
                    }
                    SignalValue::Var(ident) => {
                        if let Some(var) = self.symbols.get_mut(&ident) {
                            if let Some(sigid) = signal.id {
                                let caster = self.registors.alloc().unwrap();
                                let reg = match var.loc {
                                    Location::REG(r) => r,
                                    Location::STK(_) => todo!(),
                                    _ => {
                                        return Err(CompileError::new(
                                            CompileErrorKind::Generation(
                                                GeneratorError::NonAddressableLocation,
                                            ),
                                            Some(stmt.span),
                                        ));
                                    }
                                };

                                self.asm.reg_item(caster, &1, &Some(sigid.format()));
                                self.asm.mul::<_, String, _>(caster, None, reg);

                                self.registors.free(reg);
                                self.asm.clr(Some(reg));
                                var.loc = Location::REG(caster);
                            }

                            match var.loc {
                                Location::REG(r) => self.asm.out_reg(self.outputs.out(), r),
                                _ => continue,
                            }
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

        Ok(self.asm.finish())
    }

    fn proc_expr(&mut self, expr: &Expression) -> Result<Location, CompileErrorKind> {
        match expr {
            Expression::Value(signal) => match &signal.value {
                SignalValue::Num(n) => Ok(Location::IMM(*n)),
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

                    match var.loc {
                        Location::REG(r) => Ok(Location::REG(r)),
                        _ => todo!(),
                    }
                }
            },
            Expression::Op { lhs, rhs, op } => {
                let lhs = (lhs.deref()).clone();
                let rhs = (rhs.deref()).clone();

                let lhs_loc = self.proc_expr(&lhs).unwrap();
                let rhs_loc = self.proc_expr(&rhs).unwrap();

                self.lower_op(lhs_loc, rhs_loc, op)
            }
        }
    }

    fn lower_op(
        &mut self,
        lhs: Location,
        rhs: Location,
        op: &Sign,
    ) -> Result<Location, CompileErrorKind> {
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
