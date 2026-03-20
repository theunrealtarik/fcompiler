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
        alter: Option<Box<StatementKind>>,
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
    BinOp {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        op: BinOp,
    },
    UnaryOp {
        expr: Box<Expression>,
        op: UnaryOp,
    },
    BoolOp {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        op: CmpOp,
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

#[derive(Debug, Clone, Copy, strum_macros::Display, strum_macros::EnumIs)]
pub enum CmpOp {
    #[strum(to_string = "eq")]
    Eq,
    #[strum(to_string = "ne")]
    Ne,
    #[strum(to_string = "lt")]
    Lt,
    #[strum(to_string = "le")]
    Le,
    #[strum(to_string = "gt")]
    Gt,
    #[strum(to_string = "ge")]
    Ge,
    #[strum(to_string = "mul")]
    And,
    #[strum(to_string = "add")]
    Or,
}

impl CmpOp {
    pub fn test_op(&self) -> String {
        match self {
            Self::And | Self::Or => self.to_string(),
            cmp => format!("t{}", cmp),
        }
    }

    pub fn branch_op(&self) -> String {
        match self {
            Self::And | Self::Or => self.to_string(),
            cmp => format!("b{}", cmp),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display)]
pub enum BitOp {
    #[strum(to_string = "band")]
    BitAnd,
    #[strum(to_string = "bor")]
    BitOr,
    #[strum(to_string = "bxor")]
    BitXor,
    #[strum(to_string = "bnot")]
    BitNot,
    #[strum(to_string = "bsl")]
    ShiftLeft,
    #[strum(to_string = "bsr")]
    ShiftRight,
}

impl BitOp {
    pub fn is_commutative(&self) -> bool {
        matches!(self, Self::BitAnd | Self::BitOr | Self::BitXor)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OperationKind {
    Arithmetic(BinOp),
    Comparative(CmpOp),
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
