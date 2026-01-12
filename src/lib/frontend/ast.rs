use crate::{error::CompileError, game::SignalId};

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        ident: String,
        sigid: Option<SignalId>,
        expr: Expression,
    },
    Out(Signal),
}

#[derive(Debug, Default, Clone)]
pub struct Program(Vec<Statement>);

impl std::ops::Deref for Program {
    type Target = Vec<Statement>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Iterator for &'a Program {
    type Item = &'a Statement;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.iter().next()
    }
}

impl IntoIterator for Program {
    type Item = Statement;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<Statement>> for Program {
    fn from(value: Vec<Statement>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Value(Signal),
    Op {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        op: Sign,
    },
}

#[derive(Debug, Clone, Copy, strum_macros::Display)]
pub enum Sign {
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

#[derive(Debug, Default, Clone)]
pub struct Signal {
    pub value: SignalValue,
    pub id: Option<SignalId>,
}

impl Signal {
    pub fn new(value: SignalValue, id: Option<SignalId>) -> Self {
        Self { value, id }
    }
}

impl TryInto<i32> for Signal {
    type Error = CompileError;

    fn try_into(self) -> Result<i32, Self::Error> {
        match self.value {
            SignalValue::Num(n) => Ok(n),
            SignalValue::Var(ident) => Err(CompileError::ExpectedConstantSignal { found: ident }),
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
