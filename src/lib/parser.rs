use crate::error::CompileError;
use core::panic;

use crate::{
    ast::{Expression, Let as LetStmt, Program, Statement},
    game::Signal,
};

pub fn parse(src: &str) -> Result<Program, CompileError> {
    let mut stmts: Vec<Statement> = Vec::new();

    for (idx, line) in src.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if !line.ends_with("}") && !line.ends_with(";") {
            return Err(CompileError::MissingSemicolon(format!("(line: {})", idx)));
        }

        if line.starts_with("out(") && line.ends_with(");") {
            let inner = &line[4..line.len() - 2].trim();

            let (ident, signal_type) = inner
                .split_once(",")
                .map(|(n, t)| (n.trim(), Signal::from_str(t.trim()).ok()))
                .unwrap_or_else(|| (inner, None));

            match ident.parse::<i32>() {
                Ok(num) => stmts.push(Statement::OutNum(num, signal_type)),
                Err(_) => {
                    if validate_identifier(ident) {
                        stmts.push(Statement::OutVar(String::from(ident)));
                    } else {
                        return Err(CompileError::InvalidIdentifier(format!("(line: {})", idx)));
                    }
                }
            }

            continue;
        }

        if line.starts_with("let ") && line.ends_with(";") {
            let line = line.trim();
            let rest = &line[4..line.len() - 1];
            let (ident, expr) = rest.trim().split_once("=").unwrap();

            let (ident, signal_type) = ident
                .split_once(":")
                .map(|(n, t)| (n.trim(), Signal::from_str(t.trim()).ok()))
                .unwrap_or_else(|| (ident.trim(), None));

            if !validate_identifier(ident) {
                return Err(CompileError::InvalidIdentifier(format!("(line: {})", idx)));
            }

            let tokens = Token::tokenize(expr.trim());
            let expr = Parser::new(&tokens).parse_expression(0);

            stmts.push(Statement::Let(LetStmt::new(
                String::from(ident),
                signal_type,
                expr,
            )));
            continue;
        }

        return Err(CompileError::UnexpectedPattern(format!("(line: {})", idx)));
    }

    Ok(Program::from(stmts))
}

fn validate_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        None => return false,
        Some(c) => {
            if !(c.is_ascii_alphabetic() || c == '_') {
                return false;
            }
        }
    }

    for c in chars {
        if !(c.is_ascii_alphanumeric() || c == '_') {
            return false;
        }
    }

    true
}

// tokenizer
#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Number(i32),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    LParen,
    RParen,
    Semicolon,
}

impl Token {
    pub fn tokenize(src: &str) -> Vec<Self> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut chars = src.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' | '\n' => {
                    chars.next();
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut ident = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            ident.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Ident(ident));
                }
                '0'..='9' => {
                    let mut num = 0;
                    while let Some(&digit_ch) = chars.peek() {
                        if let Some(d) = digit_ch.to_digit(10) {
                            num = num * 10 + d as i32;
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(num));
                }
                '+' => {
                    tokens.push(Token::Plus);
                    chars.next();
                }
                '-' => {
                    tokens.push(Token::Minus);
                    chars.next();
                }
                '*' => {
                    tokens.push(Token::Star);
                    chars.next();
                }
                '/' => {
                    tokens.push(Token::Slash);
                    chars.next();
                }
                '%' => {
                    tokens.push(Token::Percent);
                    chars.next();
                }
                '=' => {
                    tokens.push(Token::Equal);
                    chars.next();
                }
                ';' => {
                    tokens.push(Token::Semicolon);
                    chars.next();
                }
                '(' => {
                    tokens.push(Token::LParen);
                    chars.next();
                }
                ')' => {
                    tokens.push(Token::RParen);
                    chars.next();
                }
                _ => panic!("unknown character"),
            }
        }

        tokens
    }

    pub fn precedence(&self) -> Option<u8> {
        match self {
            Token::Plus | Token::Minus => Some(10),
            Token::Star | Token::Slash | Token::Percent => Some(20),
            _ => None,
        }
    }
}

pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn parse_expression(&mut self, min_prec: u8) -> Expression {
        use crate::ast::Sign;
        let mut lhs = self.parse_leaf().unwrap();

        while let Some(op_token) = self.peek() {
            let prec = match op_token.precedence() {
                Some(p) if p >= min_prec => p,
                _ => break,
            };

            let sign = match op_token {
                Token::Plus => Sign::Add,
                Token::Minus => Sign::Sub,
                Token::Star => Sign::Mul,
                Token::Slash => Sign::Div,
                Token::Percent => Sign::Mod,
                _ => break,
            };

            self.next();
            let rhs = self.parse_expression(prec + 1);

            lhs = Expression::Op {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                op: sign,
            };
        }

        lhs
    }

    pub fn parse_leaf(&mut self) -> Result<Expression, CompileError> {
        match self.next() {
            Some(Token::Number(n)) => Ok(Expression::Num(*n)),
            Some(Token::Ident(name)) => Ok(Expression::Var(name.clone())),
            Some(Token::LParen) => {
                let expr = self.parse_expression(0);
                match self.next() {
                    Some(token) => {
                        if *token != Token::RParen {
                            return Err(CompileError::UnmatchedParenthesis);
                        }
                        Ok(expr)
                    }
                    None => Err(CompileError::UnmatchedParenthesis),
                }
            }
            Some(tok) => Err(CompileError::UnexpectedToken(format!("{:?}", tok))),
            None => Err(CompileError::UnexpectedPattern(String::new())),
        }
    }
}

impl<'a> std::iter::Iterator for Parser<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }
}
