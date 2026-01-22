use crate::error::*;

// Meta

#[derive(Debug, Default, Clone, Copy)]
pub struct Span {
    pub line: usize,
}

impl Span {
    pub fn new(line: usize) -> Self {
        Self { line }
    }
}

pub trait IntoOptSpan {
    fn into_opt_span(self) -> Option<Span>;
}

impl IntoOptSpan for Span {
    fn into_opt_span(self) -> Option<Span> {
        Some(self)
    }
}

impl IntoOptSpan for Option<Span> {
    fn into_opt_span(self) -> Option<Span> {
        self
    }
}

// Statements

pub type Block = Vec<StatementContext>;

#[derive(Debug, Clone)]
pub enum StatementKind {
    Block {
        body: Block,
    },
    If {
        cond: Expression,
        then: Box<StatementKind>,
        r#else: Box<StatementKind>,
    },
    Declare {
        ident: String,
        sigid: Option<crate::game::SignalId>,
        expr: Expression,
    },
    Assign {
        ident: String,
        expr: Expression,
    },
    Out(Signal),
}

#[derive(Debug, Clone)]
pub struct StatementContext {
    pub kind: StatementKind,
    pub span: Span,
}

impl StatementContext {
    pub fn new(kind: StatementKind, span: Span) -> Self {
        Self { kind, span }
    }
}

// Program

#[derive(Debug, Default, Clone)]
pub struct Program(Vec<StatementContext>);

impl std::ops::Deref for Program {
    type Target = Vec<StatementContext>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Iterator for &'a Program {
    type Item = &'a StatementContext;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.iter().next()
    }
}

impl IntoIterator for Program {
    type Item = StatementContext;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<StatementContext>> for Program {
    fn from(value: Vec<StatementContext>) -> Self {
        Self(value)
    }
}

// Expression

#[derive(Debug, Clone)]
pub enum Expression {
    Value(Signal),
    Op {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        op: BinOp,
    },
    UnaryOp {
        expr: Box<Expression>,
        op: UnaryOp,
    },
}

// Operations

#[derive(Debug, Clone, Copy, strum_macros::Display)]
pub enum BinOp {
    #[strum(to_string = "add")]
    Add,
    #[strum(to_string = "sub")]
    Sub,
    #[strum(to_string = "mul")]
    Mul,
    #[strum(to_string = "div")]
    Div,
    #[strum(to_string = "mod")]
    Mod,
}

impl BinOp {
    pub fn is_commutative(&self) -> bool {
        matches!(self, Self::Add | Self::Mul)
    }
}

#[derive(Debug, Clone, Copy, strum_macros::Display)]
pub enum UnaryOp {
    Neg,
    Not,
}

// Signals

#[derive(Debug, Clone)]
pub enum SignalValue {
    Num(i32),
    Var(String),
}

impl Default for SignalValue {
    fn default() -> Self {
        Self::Num(0)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Signal {
    pub value: SignalValue,
    pub id: Option<crate::game::SignalId>,
}

impl Signal {
    pub fn new(value: SignalValue, id: Option<crate::game::SignalId>) -> Self {
        Self { value, id }
    }
}

impl TryInto<i32> for Signal {
    type Error = CompileError;

    fn try_into(self) -> Result<i32, Self::Error> {
        match self.value {
            SignalValue::Num(n) => Ok(n),
            SignalValue::Var(_) => Err(CompileError::new(
                CompileErrorKind::Parse(ParseError::UnexpectedVariant),
                None,
            )),
        }
    }
}

impl From<i32> for Signal {
    fn from(value: i32) -> Self {
        Self {
            value: SignalValue::Num(value),
            id: None,
        }
    }
}

impl From<String> for Signal {
    fn from(v: String) -> Self {
        Self {
            value: SignalValue::Var(v),
            id: None,
        }
    }
}
