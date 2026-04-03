mod emit;
mod ir;
mod label;

pub use emit::*;
pub use ir::*;
pub use label::*;

use std::collections::HashMap;
use std::ops::Deref;

use super::mem::*;
use super::symbol::*;

use crate::error::*;
use crate::frontend::ast::*;

use crate::log;

#[derive(Default)]
pub struct Assembler {
    instr: Vec<Instruction>,
    code: String,
    program: Program,
    scopes: ScopeArena,
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

    pub fn code(&self) -> &String {
        &self.code
    }

    pub fn finish(&mut self) -> Result<&str, CompileError> {
        let stmts = self.program.to_vec();
        let global = self.scopes.enter_scope(None);
        self.handle_statements(stmts, global.clone())?;

        let instructions = self.instr.clone();
        let mut tmps = Self::count_temp(&instructions);

        self.code.push_str(&AsmFormatter::clr::<String>(None));
        for instr in instructions.iter() {
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
                Instruction::TestType { .. } => todo!(),
                Instruction::Label { name } => AsmFormatter::label(name.to_string()),
                Instruction::Jump { addr, offset } => AsmFormatter::jmp(
                    addr,
                    if let Some(offset) = offset {
                        offset.to_string()
                    } else {
                        "".to_string()
                    },
                ),
            };

            self.code.push_str(&asm_line);
            for src in instr.sources() {
                if let Operand::Temp(temp_id) = src
                    && let Some(count) = tmps.get_mut(temp_id)
                {
                    *count -= 1;
                    if *count == 0
                        && let Some(_reg) = self.memory.temps.remove(temp_id)
                    {
                        // self.code.push_str(&AsmFormatter::clr(Some(reg))); // temp fix: using regs marked as free
                        // self.memory.free(reg);
                        // log::warn!("Freed {:?} ← {:?}", reg, temp_id);
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

        Ok(&self.code)
    }

    fn ensure_location(&mut self, opr: &Operand) -> Result<Resolved, CompileError> {
        match opr {
            Operand::Persistent(symbol_id) => match self.scopes.snatch(symbol_id) {
                Some(sym) => {
                    let sym = sym.borrow();
                    Ok(Resolved::Reg(sym.loc.into()))
                }
                None => Err(CompileError::new(
                    CompileErrorKind::Generation(GeneratorError::NonAddressableSymbol),
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

    fn handle_statements(
        &mut self,
        stmts: Vec<StatementContext>,
        scope: SharedScope,
    ) -> Result<(), CompileError> {
        for stmt in stmts {
            self.handle_statement(stmt, scope.clone())?;
        }

        self.scopes.leave_scope(|scope| {
            let scope = scope.borrow();
            let locals = scope.locals.borrow();

            if scope.metadata.kind.is_global() {
                return;
            }

            log::warn!(" {:?}:{}", scope.metadata.kind, scope.metadata.idx());

            for sym in locals.values() {
                match sym.borrow().loc {
                    Location::Reg(reg) => {
                        self.memory.free(reg);
                    }
                    Location::Stk(_) => todo!(),
                }
            }
        });

        Ok(())
    }

    fn handle_statement(
        &mut self,
        stmt: StatementContext,
        current: SharedScope,
    ) -> Result<(), CompileError> {
        let current = current.borrow();

        match stmt.kind {
            StatementKind::Declare { ident, sigid, expr } => {
                let reg = self
                    .memory
                    .alloc()
                    .map_err(|k| CompileError::new(k, Some(stmt.span)))?;
                let var = Symbol::new(ident, Location::Reg(reg), sigid);

                let dst = Operand::persistent();
                let opr = self
                    .proc_expr(&expr, Some(dst))
                    .map_err(|k| CompileError::new(k, Some(stmt.span)))?;

                self.scopes.define_symbol(SymbolId(dst.into()), var);

                if dst != opr {
                    if let Some(signal_id) = sigid
                        && opr.is_imm()
                    {
                        self.mov_sig(dst, opr, signal_id);
                    } else {
                        self.mov(dst, opr);
                    }
                }
            }
            StatementKind::Assign { ident, expr } => {
                let sid = match self.scopes.lookup(&ident) {
                    Some(SymbolHandle { sid, .. }) => sid,
                    None => {
                        return Err(CompileError::new(
                            CompileErrorKind::Semantic(SemanticError::UndefinedVariable(
                                ident.to_string(),
                            )),
                            Some(stmt.span),
                        ));
                    }
                };

                let dst = Operand::Persistent(sid);
                let opr = self
                    .proc_expr(&expr, Some(dst))
                    .map_err(|k| CompileError::new(k, Some(stmt.span)))?;

                if opr != dst {
                    self.mov(dst, opr);
                }
            }
            StatementKind::Out(signal) => match signal.value {
                SignalValue::Num(scalar) => {
                    self.out(Operand::Imm(scalar), signal.id);
                }
                SignalValue::Var(ident) => {
                    if let Some(SymbolHandle { sid, .. }) = self.scopes.lookup(&ident) {
                        let target = Operand::Persistent(sid);

                        if let Some(signal_id) = signal.id {
                            self.cast(target, signal_id);
                        }

                        self.out(target, signal.id);
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
            StatementKind::If { cond, then, alter } => {
                let label = Label::raw(Label::COND);
                let label_suffixed = label.suffix(if alter.is_some() { "then" } else { "end" });

                let dst = self
                    .proc_expr(&cond, None)
                    .map_err(|k| CompileError::new(k, Some(stmt.span)))?;

                let is_singular_instr = then.len() == 1;
                if then.is_empty() {
                    return Ok(());
                }

                let then_scope = self.scopes.enter_scope_explicit(
                    Some(current.metadata.idx()),
                    ScopeMetadataBuilder::default()
                        .kind(ScopeKind::Then)
                        .build()
                        .unwrap(),
                );

                if is_singular_instr {
                    self.test_ne(dst, Operand::Imm(0));
                    self.handle_statements(then, then_scope.clone())?;
                    self.jump(Label::new(LabelKind::Ipt), Some(1));
                } else {
                    self.br_ne(dst, Operand::Imm(1), label_suffixed.clone());
                    self.handle_statements(then, then_scope.clone())?;
                    if alter.is_some() {
                        self.jump(label.suffix("end"), None);
                    }

                    self.label(label_suffixed.clone());
                }

                if let Some(alter) = alter {
                    if alter.is_empty() {
                        return Ok(());
                    }

                    let else_scope = self.scopes.enter_scope_explicit(
                        Some(current.metadata.idx()),
                        ScopeMetadataBuilder::default()
                            .kind(ScopeKind::Else)
                            .build()
                            .unwrap(),
                    );
                    self.handle_statements(alter, else_scope.clone())?;
                    if !is_singular_instr {
                        self.label(label.suffix("end"))
                    }
                }
            }
            StatementKind::Loop { body } => {
                let label = Label::raw(Label::LOOP);
                let loop_start = label.suffix("start");
                let loop_end = label.suffix("finish");

                let loop_scope = self.scopes.enter_scope_explicit(
                    Some(current.metadata.idx()),
                    ScopeMetadataBuilder::default()
                        .kind(ScopeKind::Loop)
                        .exit_label(loop_end.clone())
                        .build()
                        .unwrap(),
                );

                self.label(loop_start.clone());
                self.handle_statements(body, loop_scope.clone())?;
                self.jump(loop_start, None);
                self.label(loop_end);
            }
            StatementKind::While { .. } => todo!(),
            StatementKind::Block { body } => {
                let local_scope = self.scopes.enter_scope(Some(current.metadata.idx()));
                self.handle_statements(body, local_scope.clone())?
            }
            StatementKind::Break => {
                let ladder = self.scopes.ladder(current.metadata.idx());
                for scope in ladder {
                    // if let Some(label) = exit_label {
                    //     self.jump(label, None);
                    //     break;
                    // }
                }
            }
            _ => unimplemented!(),
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
                SignalValue::Var(ident) => match self.scopes.lookup(ident) {
                    Some(SymbolHandle { sid, .. }) => Ok(Operand::Persistent(sid)),
                    None => Err(CompileErrorKind::Semantic(
                        SemanticError::UndefinedVariable(ident.to_string()),
                    )),
                },
            },
            Expression::BinOp { lhs, rhs, op } => {
                let lhs = (lhs.deref()).clone();
                let rhs = (rhs.deref()).clone();

                let lhs_opr = self.proc_expr(&lhs, None)?;
                let rhs_opr = self.proc_expr(&rhs, None)?;

                let dst = p_dst.unwrap_or_else(Operand::temp);
                self.fold_arith_op(dst, lhs_opr, rhs_opr, op)
            }
            Expression::UnaryOp { expr, op } => match op {
                UnaryOp::Neg => {
                    let dst = p_dst.unwrap_or(Operand::temp());
                    let opr = self.proc_expr(expr, Some(dst))?;

                    if let Operand::Imm(n) = opr {
                        return Ok(Operand::Imm(-n));
                    } else {
                        self.neg(dst, opr);
                    }

                    Ok(dst)
                }
                UnaryOp::Not => {
                    let dst = p_dst.unwrap_or(Operand::temp());
                    let src = self.proc_expr(expr, Some(dst))?;
                    self.not(dst, src);
                    Ok(dst)
                }
            },
            Expression::BoolOp { lhs, rhs, op } => {
                let lhs = (lhs.deref()).clone();
                let rhs = (rhs.deref()).clone();

                let lhs_opr = self.proc_expr(&lhs, None)?;
                let rhs_opr = self.proc_expr(&rhs, None)?;

                let dst = p_dst.unwrap_or_else(Operand::temp);
                self.fold_cmp_op(dst, lhs_opr, rhs_opr, op)
            }
        }
    }

    fn fold_arith_op(
        &mut self,
        dst: Operand,
        lhs: Operand,
        rhs: Operand,
        op: &BinOp,
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
                    return self.fold_arith_op(dst, rhs, Operand::Imm(n), op);
                }
                self.emit_binop(op, dst, Some(lhs), rhs);
                Ok(dst)
            }

            // X: R OP R
            // X: R OP N
            _ => {
                self.emit_binop(op, dst, Some(lhs), rhs);
                Ok(dst)
            }
        }
    }

    fn emit_binop(&mut self, op: &BinOp, dst: Operand, src: Option<Operand>, val: Operand) {
        let src = match src {
            Some(opr) => opr,
            None => dst,
        };

        match op {
            BinOp::Add => {
                if let Operand::Imm(n) = val
                    && n == 1
                    && (dst == src)
                {
                    self.inc(dst);
                } else {
                    self.add(dst, src, val);
                }
            }
            BinOp::Sub => {
                if let Operand::Imm(n) = val
                    && n == 1
                    && (dst == src)
                {
                    self.dec(dst);
                } else {
                    self.sub(dst, src, val);
                }
            }
            BinOp::Mul => self.mul(dst, src, val),
            BinOp::Div => self.div(dst, src, val),
            BinOp::Mod => self.modu(dst, src, val),
        }
    }

    fn fold_cmp_op(
        &mut self,
        dst: Operand,
        lhs: Operand,
        rhs: Operand,
        op: &CmpOp,
    ) -> Result<Operand, CompileErrorKind> {
        match (lhs, rhs) {
            (Operand::Imm(a), Operand::Imm(b)) => {
                let r = match op {
                    CmpOp::Eq => a == b,
                    CmpOp::Ne => a != b,
                    CmpOp::Lt => a < b,
                    CmpOp::Le => a <= b,
                    CmpOp::Gt => a > b,
                    CmpOp::Ge => a >= b,
                    CmpOp::And => a * b != 0,
                    CmpOp::Or => a + b != 0,
                };

                Ok(Operand::Imm(r.into()))
            }
            _ => {
                self.emit_cmp_op(op, dst, lhs, rhs);
                Ok(dst)
            }
        }
    }

    fn emit_cmp_op(&mut self, op: &CmpOp, dst: Operand, a: Operand, b: Operand) {
        match op {
            CmpOp::Eq => self.test_eq(a, b),
            CmpOp::Ne => self.test_ne(a, b),
            CmpOp::Lt => self.test_lt(a, b),
            CmpOp::Le => self.test_le(a, b),
            CmpOp::Gt => self.test_gt(a, b),
            CmpOp::Ge => self.test_ge(a, b),
            CmpOp::And => self.mul(dst, a, b),
            CmpOp::Or => self.add(dst, a, b),
        }

        if !(op.is_and() || op.is_or()) {
            self.mov(dst, Operand::Imm(1));
        }
    }

    fn cast(&mut self, target: Operand, signal_id: crate::game::SignalId) {
        let caster = Operand::temp();
        self.mov_sig(caster, Operand::Imm(1), signal_id);
        self.mul(caster, target, target);
        self.mov(target, caster);
    }
}
