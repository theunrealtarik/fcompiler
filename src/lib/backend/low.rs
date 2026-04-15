use crate::error::*;
use crate::frontend::ast::*;

use super::ir::*;
use super::mem::*;
use super::symbol::*;
use super::tags::*;

#[allow(unused_imports)]
use crate::log;

#[derive(Debug, Default)]
pub struct Lowerer {
    tape: Tape,
    program: Program,
    scopes: ScopeArena,
    memory: MemoryManager,
}

impl Lowerer {
    pub fn new(program: Program) -> Self {
        Self {
            program,
            ..Default::default()
        }
    }

    pub fn scopes(&self) -> &ScopeArena {
        &self.scopes
    }

    pub fn tape(&self) -> &Tape {
        &self.tape
    }

    pub fn memory(&self) -> &MemoryManager {
        &self.memory
    }

    pub fn resolve(&mut self) -> Result<(), CompileError> {
        let stmts = self.program.to_vec();
        let global = self.scopes.enter_scope(None);
        self.handle_statements(stmts, global.clone())?;
        Ok(())
    }

    fn handle_statements(
        &mut self,
        stmts: Vec<StatementContext>,
        scope: SharedScope,
    ) -> Result<(), CompileError> {
        for stmt in stmts {
            self.handle_statement(stmt, scope.clone())?;
        }

        Ok(())
    }

    fn handle_statement(
        &mut self,
        stmt: StatementContext,
        parent: SharedScope,
    ) -> Result<(), CompileError> {
        match stmt.kind {
            StatementKind::Declare { ident, sigid, expr } => {
                let dst = Operand::persistent();
                let reg = self
                    .memory
                    .alloc()
                    .map_err(|k| CompileError::new(k, Some(stmt.span)))?;

                self.scopes.define_symbol(
                    SymbolId(dst.into()),
                    Symbol::new(ident.clone(), Location::Reg(reg), sigid),
                );

                let opr = self
                    .proc_expr(&expr, Some(dst))
                    .map_err(|k| CompileError::new(k, Some(stmt.span)))?;

                if dst != opr {
                    if let Some(signal_id) = sigid
                        && opr.is_imm()
                    {
                        self.tape.mov_sig(dst, opr, signal_id);
                    } else {
                        self.tape.mov(dst, opr);
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
                    self.tape.mov(dst, opr);
                }
            }
            StatementKind::Out(signal) => match signal.value {
                Literal::Integer(scalar) => {
                    self.tape.out(Operand::Imm(scalar), signal.id);
                }
                Literal::Bool(bool) => {
                    let scalar = bool as i32;
                    self.tape.out(Operand::Imm(scalar), signal.id);
                }
                Literal::Ident(ident) => {
                    if let Some(SymbolHandle { sid, .. }) = self.scopes.lookup(&ident) {
                        let target = Operand::Persistent(sid);

                        if let Some(signal_id) = signal.id {
                            self.cast(target, signal_id);
                        }

                        self.tape.out(target, signal.id);
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
                    Some(parent.metadata.idx()),
                    ScopeMetadataBuilder::default()
                        .kind(ScopeKind::Then)
                        .build()
                        .unwrap(),
                );

                let then_scope_idx = then_scope.metadata.idx();
                self.tape.event_scope_enter(then_scope_idx);

                if is_singular_instr && !then.first().map(|i| i.kind.is_out()).unwrap_or_default() {
                    self.tape.test_ne(dst, Operand::Imm(0));
                    self.handle_statements(then, then_scope.clone())?;
                } else {
                    self.tape
                        .br_eq(dst, Operand::Imm(0), label_suffixed.clone());
                    self.handle_statements(then, then_scope)?;
                    if alter.is_some() {
                        self.tape.jump(label.suffix("end"), None);
                    }

                    self.tape.label(label_suffixed.clone());
                }

                self.tape.event_scope_drop(then_scope_idx);
                self.scopes.leave_current();

                if let Some(alter) = alter {
                    if alter.is_empty() {
                        return Ok(());
                    }

                    let else_scope = self.scopes.enter_scope_explicit(
                        Some(parent.metadata.idx()),
                        ScopeMetadataBuilder::default()
                            .kind(ScopeKind::Else)
                            .build()
                            .unwrap(),
                    );
                    let else_scope_idx = else_scope.metadata.idx();

                    self.handle_statements(alter, else_scope)?;
                    if !is_singular_instr {
                        self.tape.label(label.suffix("end"))
                    }

                    self.tape.event_scope_drop(else_scope_idx);
                    self.scopes.leave_current();
                }
            }
            StatementKind::Loop { body } => {
                let loop_label = Label::raw(Label::LOOP);

                let loop_scope = self.scopes.enter_scope_explicit(
                    Some(parent.metadata.idx()),
                    ScopeMetadataBuilder::default()
                        .kind(ScopeKind::Loop)
                        .exit_label(loop_label.end())
                        .build()
                        .unwrap(),
                );
                let loop_scope_idx = loop_scope.metadata.idx();

                self.tape.event_scope_enter(loop_scope_idx);
                self.tape.label(loop_label.init());
                self.handle_statements(body, loop_scope)?;
                self.tape.jump(loop_label.init(), None);
                self.tape.label(loop_label.end());
                self.tape.event_scope_drop(loop_scope_idx);
                self.scopes.leave_current();
            }

            StatementKind::While { cond, body } => {
                let while_label = Label::raw(Label::WHILE);
                let while_scope = self.scopes.enter_scope_explicit(
                    Some(parent.metadata.idx()),
                    ScopeMetadataBuilder::default()
                        .kind(ScopeKind::While)
                        .exit_label(while_label.end())
                        .build()
                        .unwrap(),
                );

                let while_scope_idx = while_scope.metadata.idx();
                self.tape.event_scope_enter(while_scope_idx);
                self.tape.label(while_label.init());
                self.handle_statements(body, while_scope)?;

                let dst = self
                    .proc_expr(&cond, None)
                    .map_err(|k| CompileError::new(k, Some(stmt.span)))?;

                self.tape.test_eq(dst, Operand::Imm(1));
                self.tape.jump(while_label.init(), None);
                self.tape.label(while_label.end());

                self.tape.event_scope_drop(while_scope_idx);
                self.scopes.leave_current();
            }

            StatementKind::For { iter, range, body } => {
                let for_label = Label::raw(Label::FOR);
                let for_scope = self.scopes.enter_scope_explicit(
                    Some(parent.metadata.idx()),
                    ScopeMetadataBuilder::default()
                        .start_label(for_label.init())
                        .kind(ScopeKind::For)
                        .exit_label(for_label.end())
                        .build()
                        .unwrap(),
                );
                let for_scope_idx = for_scope.metadata.idx();
                let contains_break = body.iter().find(|instr| instr.kind.is_break()).is_some();

                self.tape.event_scope_enter(for_scope_idx);

                let dst = Operand::persistent();
                let reg = self
                    .memory
                    .alloc()
                    .map_err(|k| CompileError::new(k, Some(stmt.span)))?;

                self.scopes.define_symbol(
                    SymbolId(dst.into()),
                    Symbol::new(iter.clone(), Location::Reg(reg), None),
                );

                let start = self
                    .eval_literal(&range.start)
                    .map_err(|kind| CompileError::new(kind, None))?;
                let end = self
                    .eval_literal(&range.end)
                    .map_err(|kind| CompileError::new(kind, None))?;

                self.tape.mov(dst, start);
                self.tape.label(for_label.init());

                self.handle_statements(body, for_scope)?;
                self.tape.inc(dst);

                if range.inclusive {
                    self.tape.br_le(dst, end, for_label.init());
                } else {
                    self.tape.br_lt(dst, end, for_label.init());
                }

                if contains_break {
                    self.tape.label(for_label.end());
                }

                self.tape.event_scope_drop(for_scope_idx);
                self.scopes.leave_current();
            }

            StatementKind::Block { body } => {
                let local_scope = self.scopes.enter_scope(Some(parent.metadata.idx()));
                let local_scope_idx = local_scope.metadata.idx();
                self.tape.event_scope_enter(local_scope_idx);
                self.handle_statements(body, local_scope)?;
                self.tape.event_scope_drop(local_scope_idx);
                self.scopes.leave_current();
            }
            StatementKind::Break => {
                let current = self.scopes.current().unwrap();
                let ladder = self.scopes.ladder(current);

                log::info!("{:#?}", ladder);

                for scope in ladder {
                    let metadata = scope.metadata;
                    if metadata.kind.is_breakable()
                        && let Some(exit_label) = metadata.exit_label
                    {
                        self.tape.jump(exit_label, None);
                        return Ok(());
                    }
                }

                return Err(gen_err!(
                    GeneratorError::InvalidInstruction {
                        msg: String::from("break outside of loop")
                    },
                    stmt.span
                ));
            }
            _ => unimplemented!(),
        }

        Ok(())
    }

    fn eval_literal(&self, literal: &Literal) -> Result<Operand, CompileErrorKind> {
        match literal {
            Literal::Integer(n) => Ok(Operand::immediate(*n)),
            Literal::Bool(b) => Ok(Operand::immediate(*b as i32)),
            Literal::Ident(ident) => match self.scopes.lookup(ident) {
                Some(SymbolHandle { sid, .. }) => Ok(Operand::Persistent(sid)),
                None => Err(CompileErrorKind::Semantic(
                    SemanticError::UndefinedVariable(ident.to_string()),
                )),
            },
        }
    }

    fn proc_expr(
        &mut self,
        expr: &Expression,
        p_dst: Option<Operand>,
    ) -> Result<Operand, CompileErrorKind> {
        match expr {
            Expression::Value(signal) => self.eval_literal(&signal.value),
            Expression::BinOp { lhs, rhs, op } => {
                let lhs = (*lhs).clone();
                let rhs = (*rhs).clone();

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
                        self.tape.neg(dst, opr);
                    }

                    Ok(dst)
                }
                UnaryOp::Not => {
                    let dst = p_dst.unwrap_or(Operand::temp());
                    let src = self.proc_expr(expr, Some(dst))?;
                    self.tape.not(dst, src);
                    Ok(dst)
                }
            },
            Expression::BoolOp { lhs, rhs, op } => {
                let lhs = (*lhs).clone();
                let rhs = (*rhs).clone();

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
                    self.tape.inc(dst);
                } else {
                    self.tape.add(dst, src, val);
                }
            }
            BinOp::Sub => {
                if let Operand::Imm(n) = val
                    && n == 1
                    && (dst == src)
                {
                    self.tape.dec(dst);
                } else {
                    self.tape.sub(dst, src, val);
                }
            }
            BinOp::Mul => self.tape.mul(dst, src, val),
            BinOp::Div => self.tape.div(dst, src, val),
            BinOp::Mod => self.tape.modu(dst, src, val),
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
        let is_conjunction = op.is_and() || op.is_or();
        if !is_conjunction {
            self.tape.event_free(vec![dst]);
        }

        match op {
            CmpOp::Eq => self.tape.test_eq(a, b),
            CmpOp::Ne => self.tape.test_ne(a, b),
            CmpOp::Lt => self.tape.test_lt(a, b),
            CmpOp::Le => self.tape.test_le(a, b),
            CmpOp::Gt => self.tape.test_gt(a, b),
            CmpOp::Ge => self.tape.test_ge(a, b),
            CmpOp::And => self.tape.mul(dst, a, b),
            CmpOp::Or => self.tape.add(dst, a, b),
        }

        if !is_conjunction {
            self.tape.mov(dst, Operand::Imm(1));
        }
    }

    fn cast(&mut self, target: Operand, signal_id: crate::game::SignalId) {
        let caster = Operand::temp();
        self.tape.mov_sig(caster, Operand::Imm(1), signal_id);
        self.tape.mul(caster, caster, target);
        self.tape.mov(target, caster);
    }
}
