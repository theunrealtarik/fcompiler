use super::ast::*;
use crate::error::*;
use crate::game::SignalId;

pub fn parse(src: &str) -> Result<Program, CompileError> {
    let mut stmts: Vec<StatementContext> = Vec::new();

    for (idx, line) in src.lines().enumerate() {
        let line = line.trim();
        let line_span = Span::new(idx);

        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if !line.ends_with("}") && !line.ends_with(";") {
            return Err(CompileError::new(
                CompileErrorKind::Parse(ParseError::MissingSemicolon),
                Some(line_span),
            ));
        }

        if line.starts_with("out(") && line.ends_with(");") {
            let inner = &line[4..line.len() - 2].trim();

            let (ident, signal_type) = inner
                .split_once(",")
                .map(|(n, t)| (n.trim(), SignalId::from_str(t.trim()).ok()))
                .unwrap_or_else(|| (inner, None));

            let mut signal = Signal::default();
            match ident.parse::<i32>() {
                Ok(num) => {
                    if signal_type.is_none() {
                        return Err(CompileError::new(
                            CompileErrorKind::Parse(ParseError::MissingSignalType),
                            Some(line_span),
                        ));
                    }

                    signal = Signal::from(num);
                }
                Err(_) => {
                    if validate_identifier(ident) {
                        signal.value = SignalValue::Var(ident.to_string());
                    } else {
                        return Err(CompileError::new(
                            CompileErrorKind::Parse(ParseError::InvalidIdentifier),
                            Some(line_span),
                        ));
                    }
                }
            }

            signal.id = signal_type;
            stmts.push(StatementContext::new(StatementKind::Out(signal), line_span));
            continue;
        }

        if line.starts_with("let ") && line.ends_with(";") {
            let line = line.trim();
            let rest = &line[4..line.len() - 1];
            let (ident, expr) = rest.trim().split_once("=").unwrap();

            let (ident, signal_id) = ident
                .split_once(":")
                .map(|(n, t)| (n.trim(), SignalId::from_str(t.trim()).ok()))
                .unwrap_or_else(|| (ident.trim(), None));

            if !validate_identifier(ident) {
                return Err(CompileError::new(
                    CompileErrorKind::Parse(ParseError::InvalidIdentifier),
                    Some(line_span),
                ));
            }

            let tokens = Token::tokenize(expr.trim());
            let mut parser = Lexer::new(&tokens);
            let expr = match parser.parse_expression(0) {
                Ok(e) => e,
                Err(k) => return Err(CompileError::new(k, Some(line_span))),
            };

            if !parser.is_eof() {
                return Err(CompileError::new(
                    CompileErrorKind::Lex(LexerError::UnexpectedEndOfInput),
                    Some(line_span),
                ));
            }

            stmts.push(StatementContext::new(
                StatementKind::Let {
                    ident: String::from(ident),
                    sigid: signal_id,
                    expr,
                },
                line_span,
            ));
            continue;
        }

        return Err(CompileError::new(
            CompileErrorKind::Parse(ParseError::UnexpectedPattern),
            Some(line_span),
        ));
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

                    while let Some(ch_digit) = chars.peek() {
                        if let Some(d) = ch_digit.to_digit(10) {
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

#[derive(Debug)]
pub struct Lexer<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn parse_expression(&mut self, min_prec: u8) -> Result<Expression, CompileErrorKind> {
        let mut lhs = self.parse_leaf()?;

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
            let rhs = self.parse_expression(prec + 1)?;

            lhs = Expression::Op {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                op: sign,
            };
        }

        Ok(lhs)
    }

    pub fn parse_leaf(&mut self) -> Result<Expression, CompileErrorKind> {
        match self.next() {
            Some(Token::Number(n)) => Ok(Expression::Value(Signal::from(*n))),
            Some(Token::Ident(name)) => Ok(Expression::Value(Signal::from(name.to_string()))),
            Some(Token::LParen) => {
                let expr = self.parse_expression(0);
                match self.next() {
                    Some(token) => {
                        if *token != Token::RParen {
                            return Err(CompileErrorKind::Lex(LexerError::UnmatchedParenthesis));
                        }
                        expr
                    }
                    None => Err(CompileErrorKind::Lex(LexerError::UnmatchedParenthesis)),
                }
            }
            Some(Token::Minus) => {
                match self.parse_expression(Token::Minus.precedence().unwrap() + 1) {
                    Ok(expr) => Ok(Expression::Op {
                        lhs: Box::new(Expression::Value(Signal::from(0))),
                        rhs: Box::new(expr),
                        op: Sign::Sub,
                    }),
                    Err(k) => Err(k),
                }
            }
            Some(tok) => Err(CompileErrorKind::Lex(LexerError::InvalidExpression(
                format!("{:?}", tok),
            ))),
            None => Err(CompileErrorKind::Parse(ParseError::UnexpectedPattern)),
        }
    }
}

impl<'a> std::iter::Iterator for Lexer<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }
}
