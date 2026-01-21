use std::str::FromStr;

use super::ast::*;
use super::lexemes;
use crate::error::*;
use crate::game::SignalId;

#[derive(Default, Debug)]
pub struct Parser {
    cursor: usize,
    tokens: Vec<TokenContext>,
    program: Program,
}

#[allow(dead_code)]
impl Parser {
    pub fn new(src: &str) -> Result<Self, CompileError> {
        let mut tokens = Token::tokenize(src)?;
        tokens.push(TokenContext::from(Token::EOF));

        Ok(Self {
            cursor: 0,
            tokens,
            program: Program::default(),
        })
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn parse(&mut self) -> Result<Vec<StatementContext>, CompileError> {
        self.parse_until(Token::EOF)
    }

    fn parse_until(&mut self, limit: Token) -> Result<Vec<StatementContext>, CompileError> {
        let mut stmts: Vec<StatementContext> = Vec::new();

        while let Some(TokenContext { kind, span }) = self.peek_context()
            && kind != &limit
        {
            let span = span.clone();
            let stmt = match kind {
                Token::Let => self.parse_declaration(),
                Token::Ident { name, sid } => {
                    if let Some(t) = sid {
                        return Err(parse_err!(
                            ParseError::UnexpectedToken {
                                found: t.to_string(),
                            },
                            span
                        ));
                    }

                    let name = name.clone();
                    self.parse_assignment(&name)
                }
                Token::Out => self.parse_out(),
                Token::LCurly => self.parse_block(),
                _ => {
                    return Err(parse_err!(
                        ParseError::UnexpectedToken {
                            found: kind.to_string(),
                        },
                        span
                    ));
                }
            };

            stmts.push(StatementContext::new(stmt?, span.unwrap_or_default()));
        }

        Ok(stmts)
    }

    fn parse_declaration(&mut self) -> Result<StatementKind, CompileError> {
        self.expect(Token::Let)?;

        let token = self.consume()?;
        if let Token::Ident { name, sid } = token.kind {
            self.expect(Token::Equal)?;

            if lexemes::RESERVED_KEYWORDS.contains(&name.as_str()) {
                return Err(parse_err!(
                    ParseError::ReservedKeyword {
                        keyword: name.to_string(),
                    },
                    token.span
                ));
            }

            if !self.validate_identifier(&name) {
                return Err(parse_err!(ParseError::InvalidIdentifier, token.span));
            }

            let signal_id = match sid {
                Some(id) => SignalId::from_str(&id).ok(),
                None => None,
            };

            let expr = Expresso::new(&self.consume_until(Token::Semicolon)?).parse_expression(0)?;
            return Ok(StatementKind::Declare {
                ident: name,
                sigid: signal_id,
                expr,
            });
        }

        Err(parse_err!(ParseError::UnexpectedPattern, token.span))
    }

    fn parse_assignment(&mut self, ident: &String) -> Result<StatementKind, CompileError> {
        self.expect(Token::Ident {
            name: ident.to_string(),
            sid: None,
        })?;
        self.expect(Token::Equal)?;

        let tokens = self.consume_until(Token::Semicolon)?;
        let expr = Expresso::new(&tokens).parse_expression(0)?;
        Ok(StatementKind::Assign {
            ident: ident.to_string(),
            expr,
        })
    }

    fn parse_out(&mut self) -> Result<StatementKind, CompileError> {
        self.expect(Token::Out)?;
        self.expect(Token::LParen)?;

        let mut signal = Signal::default();
        let token = self.consume()?;

        match token.kind {
            Token::Ident { name, sid } => {
                signal.value = SignalValue::Var(name);
                signal.id = match sid {
                    Some(id) => SignalId::from_str(&id).ok(),
                    None => None,
                };
            }
            Token::Number(n) => {
                signal.value = SignalValue::Num(n);
            }
            _ => return Err(parse_err!(ParseError::UnexpectedPattern, token.span)),
        }
        let token = self.consume()?;
        if let Token::RParen = token.kind {
            self.expect(Token::Semicolon)?;
            Ok(StatementKind::Out(signal))
        } else {
            Err(parse_err!(
                ParseError::UnexpectedToken {
                    found: self.next().map(|t| t.to_string()).unwrap_or_default(),
                },
                token.span
            ))
        }
    }

    fn parse_block(&mut self) -> Result<StatementKind, CompileError> {
        self.expect(Token::LCurly)?;
        let body = self.parse_until(Token::RCurly)?;
        self.expect(Token::RCurly)?;

        Ok(StatementKind::Block { body })
    }

    fn peek(&self) -> Option<&Token> {
        self.peek_context().map(|tc| &tc.kind)
    }

    fn peek_context(&self) -> Option<&TokenContext> {
        self.tokens.get(self.cursor)
    }

    fn next_context(&mut self) -> Option<TokenContext> {
        match self.tokens.get(self.cursor) {
            Some(tc) => {
                self.cursor += 1;
                Some(tc.clone())
            }
            None => None,
        }
    }

    fn consume_until(&mut self, limit: Token) -> Result<Vec<TokenContext>, CompileError> {
        let mut collected = Vec::new();
        while let Some(TokenContext { kind, .. }) = self.peek_context() {
            if kind == &limit {
                self.consume()?;
                break;
            }

            let token = self.consume()?;
            collected.push(token);
        }

        Ok(collected)
    }

    fn consume(&mut self) -> Result<TokenContext, CompileError> {
        self.next_context()
            .ok_or(parse_err!(ParseError::UnexpectedPattern))
    }

    fn expect(&mut self, expected_token: Token) -> Result<(), CompileError> {
        match self.peek_context() {
            Some(token) if token.kind == expected_token => {
                self.next();
                Ok(())
            }
            Some(token) => Err(parse_err!(
                ParseError::UnexpectedToken {
                    found: token.kind.to_string(),
                },
                token.span
            )),
            None => Err(parse_err!(ParseError::UnexpectedEof, None)),
        }
    }

    fn validate_identifier(&self, s: &str) -> bool {
        let mut chars = s.chars();
        match chars.next() {
            None => return false,
            Some(c) => {
                if !(c.is_ascii_alphabetic() || c == lexemes::CH_UNDERSCORE) {
                    return false;
                }
            }
        }

        for c in chars {
            if !(c.is_ascii_alphanumeric() || c == lexemes::CH_UNDERSCORE) {
                return false;
            }
        }

        true
    }
}

impl std::iter::Iterator for Parser {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_context().map(|tc| tc.kind)
    }
}

#[derive(Clone, Debug)]
pub struct TokenContext {
    kind: Token,
    span: Option<Span>,
}

impl From<Token> for TokenContext {
    fn from(value: Token) -> Self {
        Self {
            kind: value,
            span: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, strum_macros::EnumString, strum_macros::Display, Clone)]
pub enum Token {
    // Keywords
    Let,
    Out,
    If,
    Else,
    While,

    // Literals
    Number(i32),
    Ident {
        name: String,
        sid: Option<String>,
    },
    Boolean(bool),

    // Operations
    #[strum(serialize = "+")]
    Plus,
    #[strum(serialize = "-")]
    Minus,
    #[strum(serialize = "*")]
    Star,
    #[strum(serialize = "/")]
    Slash,
    #[strum(serialize = "%")]
    Percent,
    #[strum(serialize = "=")]
    Equal,

    // Comparison
    #[strum(serialize = "<")]
    Less,
    #[strum(serialize = ">")]
    Greater,

    // Logic
    #[strum(serialize = "!")]
    Bang,
    #[strum(serialize = "&")]
    And,
    #[strum(serialize = "|")]
    Or,
    #[strum(serialize = "&&")]
    AndAnd,
    #[strum(serialize = "||")]
    OrOr,

    // Symbols
    #[strum(serialize = "(")]
    LParen,
    #[strum(serialize = ")")]
    RParen,
    #[strum(serialize = "{")]
    LCurly,
    #[strum(to_string = "}}")]
    RCurly,
    #[strum(serialize = ",")]
    Comma,
    #[strum(serialize = ":")]
    Colon,
    #[strum(serialize = ";")]
    Semicolon,

    // Formatting?
    #[strum(serialize = " ")]
    Whitespace,
    #[strum(serialize = "_")]
    Underscore,
    #[strum(serialize = "\t")]
    Tab,
    #[strum(serialize = "\n")]
    Newline,
    EOF,
}

impl Token {
    pub fn tokenize(src: &str) -> Result<Vec<TokenContext>, CompileError> {
        use lexemes::*;
        let mut tokens: Vec<TokenContext> = Vec::new();
        let mut chars = src.chars().peekable();
        let mut span = Span::new(1);

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' => {
                    chars.next();
                }
                CH_NL => {
                    chars.next();
                    span.line += 1;
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let ident = Self::collect_while(&mut chars, |c| {
                        c.is_alphanumeric() || c == lexemes::CH_UNDERSCORE
                    });

                    Self::skip_whitespace(&mut chars);

                    let mut sid: Option<String> = None;
                    if let Some(&':') = chars.peek() {
                        chars.next();

                        Self::skip_whitespace(&mut chars);
                        let type_name = Self::collect_while(&mut chars, |c| c.is_alphabetic());

                        if !type_name.is_empty() {
                            sid = Some(type_name);
                        }
                    }

                    let token = match ident.as_str() {
                        KW_LET => Token::Let,
                        KW_OUT => Token::Out,
                        KW_IF => Token::If,
                        KW_ELSE => Token::Else,
                        KW_TRUE => Token::Boolean(true),
                        KW_FALSE => Token::Boolean(false),
                        _ => Token::Ident {
                            name: ident.to_string(),
                            sid,
                        },
                    };

                    tokens.push(TokenContext {
                        kind: token,
                        span: Some(span.clone()),
                    });
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

                    tokens.push(TokenContext {
                        kind: Token::Number(num),
                        span: Some(span.clone()),
                    });
                }
                CH_LCURLY | CH_RCURLY => {
                    tokens.push(TokenContext {
                        kind: if ch == CH_LCURLY {
                            Token::LCurly
                        } else {
                            Token::RCurly
                        },
                        span: Some(span.clone()),
                    });
                    chars.next();
                }
                CH_ADD | CH_SUB | CH_MUL | CH_DIV | CH_MOD | CH_EQ | CH_SEMICOLON | CH_LPARAN
                | CH_RPARAN | CH_NOT | CH_COMMA => {
                    let t = Token::from_str(&ch.to_string()).map_err(|_| {
                        parse_err!(
                            ParseError::UnknownCharacter {
                                found: String::from(ch),
                            },
                            span
                        )
                    })?;
                    tokens.push(TokenContext {
                        kind: t,
                        span: Some(span.clone()),
                    });
                    chars.next();
                }
                CH_AND => {
                    chars.next();
                    if let Some(nx) = chars.next() {
                        match nx {
                            CH_AND => tokens.push(TokenContext {
                                kind: Token::AndAnd,
                                span: Some(span.clone()),
                            }),
                            _ => tokens.push(TokenContext {
                                kind: Token::And,
                                span: Some(span.clone()),
                            }),
                        }
                    }
                }
                CH_OR => {
                    chars.next();
                    if let Some(nx) = chars.next() {
                        match nx {
                            CH_OR => tokens.push(TokenContext {
                                kind: Token::OrOr,
                                span: Some(span.clone()),
                            }),
                            _ => tokens.push(TokenContext {
                                kind: Token::Or,
                                span: Some(span.clone()),
                            }),
                        }
                    }
                }
                c => return Err(lex_err!(LexerError::UnknownCharacter(c), span)),
            }
        }

        Ok(tokens)
    }

    pub fn precedence(&self) -> Option<u8> {
        match self {
            Token::Bang => Some(10),
            Token::Star | Token::Slash | Token::Percent => Some(9),
            Token::Plus | Token::Minus => Some(8),
            Token::AndAnd => Some(3),
            Token::OrOr => Some(2),
            _ => None,
        }
    }

    fn skip_while<I, F>(chars: &mut std::iter::Peekable<I>, pred: F)
    where
        I: Iterator<Item = char>,
        F: Fn(char) -> bool,
    {
        while let Some(&c) = chars.peek() {
            if pred(c) {
                chars.next();
            } else {
                break;
            }
        }
    }

    fn skip_whitespace<I>(chars: &mut std::iter::Peekable<I>)
    where
        I: Iterator<Item = char>,
    {
        Self::skip_while(chars, |c| c.is_whitespace());
    }

    fn collect_while<I, F>(chars: &mut std::iter::Peekable<I>, pred: F) -> String
    where
        I: Iterator<Item = char>,
        F: Fn(char) -> bool,
    {
        let mut buff = String::new();
        while let Some(&c) = chars.peek() {
            if pred(c) {
                buff.push(c);
                chars.next();
            } else {
                break;
            }
        }

        buff
    }
}

#[derive(Debug)]
pub struct Expresso<'a> {
    tokens: &'a [TokenContext],
    pos: usize,
}

impl<'a> Expresso<'a> {
    pub fn new(tokens: &'a [TokenContext]) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub fn peek(&self) -> Option<&TokenContext> {
        self.tokens.get(self.pos)
    }

    pub fn parse_expression(&mut self, min_prec: u8) -> Result<Expression, CompileError> {
        let mut lhs = self.parse_leaf()?;

        while let Some(op_token) = self.peek() {
            let prec = match op_token.kind.precedence() {
                Some(p) if p >= min_prec => p,
                _ => break,
            };

            let sign = match op_token.kind {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                Token::AndAnd => BinOp::Mul,
                Token::OrOr => BinOp::Add,
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

    pub fn parse_leaf(&mut self) -> Result<Expression, CompileError> {
        if let Some(token) = self.next() {
            let span = token.span.clone();

            match &token.kind {
                Token::Number(n) => Ok(Expression::Value(Signal::from(*n))),
                Token::Ident { name, .. } => Ok(Expression::Value(Signal::from(name.to_string()))),
                Token::Boolean(b) => Ok(Expression::Value(Signal::from(*b as i32))),
                Token::LParen => {
                    let expr = self.parse_expression(0);
                    match self.next() {
                        Some(TokenContext { kind, span }) => {
                            if *kind != Token::RParen {
                                return Err(lex_err!(LexerError::UnmatchedParenthesis, *span));
                            }
                            expr
                        }
                        None => Err(lex_err!(LexerError::UnmatchedParenthesis, span)),
                    }
                }
                Token::Minus => {
                    match self.parse_expression(Token::Minus.precedence().unwrap() + 1) {
                        Ok(expr) => Ok(Expression::UnaryOp {
                            expr: Box::new(expr),
                            op: UnaryOp::Neg,
                        }),
                        Err(k) => Err(k),
                    }
                }
                Token::Bang => match self.parse_expression(Token::Bang.precedence().unwrap()) {
                    Ok(expr) => Ok(Expression::UnaryOp {
                        expr: Box::new(expr),
                        op: UnaryOp::Not,
                    }),
                    Err(k) => Err(k),
                },
                t => Err(lex_err!(
                    LexerError::InvalidExpression(format!("{:?}", t)),
                    span
                )),
            }
        } else {
            Err(parse_err!(ParseError::UnexpectedEof))
        }
    }
}

impl<'a> std::iter::Iterator for Expresso<'a> {
    type Item = &'a TokenContext;

    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }
}
