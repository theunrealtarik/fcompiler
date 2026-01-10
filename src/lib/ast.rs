use crate::game::Signal;

#[derive(Debug, Clone)]
pub enum Statement {
    Let(Let),
    OutVar(String),
    OutNum(i32, Option<Signal>),
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
    Num(i32),
    Var(String),
    Op {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        op: Sign,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Sign {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone)]
pub struct Let {
    pub ident: String,
    pub signal: Option<Signal>,
    pub expr: Expression,
}

impl Let {
    pub fn new(name: String, signal: Option<Signal>, expr: Expression) -> Self {
        Self {
            ident: name,
            signal,
            expr,
        }
    }
}
