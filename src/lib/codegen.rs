use std::ops::Deref;

use crate::asm::*;
use crate::mem::*;

use crate::{
    ast::{Expression, Let as LetStmt, Program, Statement},
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
                    let reg = self.registors.alloc().unwrap();
                    self.gen_expr(reg, &let_stmt).unwrap();
                }
                Statement::OutNum(value, signal) => {
                    let reg = self.registors.alloc().unwrap();
                    let signal = signal.map(|s| s.format());
                    self.asm.out_item(Out(reg), &value, &signal);
                }
                Statement::OutVar(ident) => match self.symbols.get(&ident) {
                    Some(var) => {
                        if let Some(value) = var.value {
                            let signal = var.signal.map(|s| s.format());
                            self.asm.out_item(Out(var.reg), &value, &signal);
                        }
                    }
                    None => {
                        return Err(CompileError::UndefinedVariable(ident.clone()));
                    }
                },
            }
        }

        dbg!(&self.symbols);
        dbg!(&self.registors);
        Ok(self.asm.finish())
    }

    fn gen_expr(&mut self, reg: Register, let_stmt: &LetStmt) -> Result<(), CompileError> {
        let LetStmt {
            ident,
            signal,
            expr,
        } = let_stmt;

        match expr {
            Expression::Num(n) => {
                let variable = Variable::new(ident.to_string(), reg, None, Some(*n), *signal);
                self.symbols.insert(ident.clone(), variable);
                self.asm.reg_item(reg, n, &signal.map(|s| s.format()));
            }
            Expression::Var(r_ident) => match self.symbols.get(r_ident) {
                Some(var) => {
                    let dst = reg;
                    let src = var.reg;
                    self.asm.mov(dst, src);
                }
                None => {
                    return Err(CompileError::UndefinedVariable(r_ident.clone()));
                }
            },
            Expression::Op { lhs, rhs, op } => {
                let lhs_reg = reg;
                let rhs_reg = self.registors.alloc().unwrap();

                let lhs_stmt = LetStmt::new("tmp".to_string(), None, (lhs.deref()).clone());
                let rhs_stmt = LetStmt::new("tmp".to_string(), None, (rhs.deref()).clone());

                self.gen_expr(lhs_reg, &lhs_stmt).unwrap();
                self.gen_expr(rhs_reg, &rhs_stmt).unwrap();

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
            }
        }

        self.symbols.insert(
            ident.to_string(),
            Variable::new(ident.to_string(), reg, None, None, *signal),
        );

        Ok(())
    }
}
