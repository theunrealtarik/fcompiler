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
                    let reg = self.proc_expr(&expr, &sigid).unwrap();
                    self.symbols.insert(
                        ident.to_string(),
                        Variable::new(ident.to_string(), reg, sigid),
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
                                Location::STK(_) => todo!(),
                                Location::IMM(_) => todo!(),
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
            Expression::Value(signal) => {
                let reg = self.registors.alloc().unwrap();
                match &signal.value {
                    SignalValue::Num(n) => {
                        self.asm.reg_item(reg, n, &parent_sigid.map(|s| s.format()));
                    }
                    SignalValue::Var(r_ident) => {
                        let var = self
                            .symbols
                            .get(r_ident)
                            .ok_or_else(|| CompileError::UndefinedVariable(r_ident.clone()))
                            .unwrap();

                        let dst = reg;
                        let src = var.loc;
                        self.asm.mov(dst, src);
                    }
                }

                Ok(Location::REG(reg))
            }
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
            (Location::REG(dst), Location::REG(src)) => {
                match op {
                    Sign::Add => self.asm.add_r(dst, src),
                    Sign::Sub => self.asm.sub_r(dst, src),
                    Sign::Mul => self.asm.mul_r(dst, src),
                    Sign::Div => self.asm.div_r(dst, src),
                    Sign::Mod => self.asm.modu_r(dst, src),
                }
                self.registors.free(src);
                Ok(Location::REG(dst))
            }
            _ => unimplemented!("CASE NOT IMPLEMENTED"),
        }
    }
}
