use std::ops::Deref;

use crate::asm::*;
use crate::mem::*;

use crate::{
    ast::{Expression, LetStmt, Program, Statement},
    error::CompileError,
    symbol::SymbolTable,
};

#[derive(Debug, Default)]
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
                Statement::Let(let_stmt) => {
                    let LetStmt { ident, sigid, expr } = let_stmt;
                    let reg = self.proc_expr(&expr, &sigid).unwrap();
                    self.symbols.insert(
                        ident.to_string(),
                        Variable::new(ident.to_string(), reg, None, sigid),
                    );
                }
                Statement::Out(signal) => match signal.value {
                    crate::ast::SignalValue::Num(scalar) => {
                        let signal = signal.id.map(|s| s.format());
                        self.asm.out_item(self.outputs.out(), &scalar, &signal);
                    }
                    crate::ast::SignalValue::Var(ident) => {
                        if let Some(var) = self.symbols.get(&ident) {
                            match signal.id {
                                Some(sigid) => {
                                    let tmp = self.registors.alloc().unwrap();
                                    self.asm.reg_item(tmp, &1, &Some(sigid.format()));
                                    self.asm.mul(tmp, var.reg);
                                    self.asm.clr(Some(tmp));
                                }
                                None => {
                                    self.asm.out_reg(self.outputs.out(), var.reg);
                                }
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
    ) -> Result<Register, CompileError> {
        match expr {
            Expression::Value(signal) => {
                let reg = self.registors.alloc().unwrap();
                match &signal.value {
                    crate::ast::SignalValue::Num(n) => {
                        self.asm.reg_item(reg, n, &parent_sigid.map(|s| s.format()));
                    }
                    crate::ast::SignalValue::Var(r_ident) => match self.symbols.get(r_ident) {
                        Some(var) => {
                            let dst = reg;
                            let src = var.reg;
                            self.asm.mov(dst, src);
                        }
                        None => {
                            return Err(CompileError::UndefinedVariable(r_ident.clone()));
                        }
                    },
                }

                Ok(reg)
            }
            Expression::Op { lhs, rhs, op } => {
                let lhs = (lhs.deref()).clone();
                let rhs = (rhs.deref()).clone();

                let lhs_reg = self.proc_expr(&lhs, parent_sigid).unwrap();
                let rhs_reg = self.proc_expr(&rhs, parent_sigid).unwrap();

                match op {
                    crate::ast::Sign::Add => self.asm.add(lhs_reg, rhs_reg),
                    crate::ast::Sign::Sub => self.asm.sub(lhs_reg, rhs_reg),
                    crate::ast::Sign::Mul => self.asm.mul(lhs_reg, rhs_reg),
                    crate::ast::Sign::Div => self.asm.div(lhs_reg, rhs_reg),
                    crate::ast::Sign::Mod => self.asm.modu(lhs_reg, rhs_reg),
                }

                self.registors.free(rhs_reg);
                self.asm.clr(Some(rhs_reg));

                let lhs_entry = self
                    .symbols
                    .iter()
                    .find(|(_, var)| var.reg == rhs_reg)
                    .map(|(k, _)| k.clone());

                if let Some(key) = lhs_entry {
                    self.symbols.remove(&key);
                }

                Ok(lhs_reg)
            }
        }
    }
}
