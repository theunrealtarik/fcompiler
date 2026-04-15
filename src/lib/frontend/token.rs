#[derive(Clone, Debug)]
pub struct TokenContext {
    pub kind: Token,
    pub span: Option<Span>,
}

impl From<Token> for TokenContext {
    fn from(value: Token) -> Self {
        Self {
            kind: value,
            span: None,
        }
    }
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    strum_macros::EnumString,
    strum_macros::EnumIs,
    strum_macros::Display,
    Clone,
)]
pub enum Token {
    // keywords
    Let,
    Out,
    In,
    If,
    Else,
    For,
    Loop,
    While,
    Break,
    Return,
    Continue,

    // literals
    Integer(i32),
    Ident {
        name: String,
        sid: Option<String>,
    },
    Boolean(bool),

    // operations
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

    // logic
    #[strum(serialize = "!")]
    Bang,
    #[strum(serialize = "&")]
    And,
    #[strum(serialize = "|")]
    Or,
    #[strum(serialize = "^")]
    Xor,
    #[strum(serialize = "&&")]
    AndAnd,
    #[strum(serialize = "||")]
    OrOr,

    // equality
    #[strum(serialize = "==")]
    EqualEqual,
    #[strum(serialize = "!=")]
    BangEqual,

    // comparison
    #[strum(serialize = "<")]
    Lesser,
    #[strum(serialize = "<=")]
    LesserEqual,
    #[strum(serialize = ">")]
    Greater,
    #[strum(serialize = ">=")]
    GreaterEqual,

    // symbols
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

    #[strum(serialize = "..")]
    DotDot,
    #[strum(serialize = "..=")]
    DotDotEqual,

    // formatting?
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

use std::str::FromStr;

use super::ast::*;
use super::lexemes;

use crate::error::*;

impl Token {
    pub fn tokenize(src: &str) -> Result<Vec<TokenContext>, CompileError> {
        use lexemes::*;
        let mut tokens: Vec<TokenContext> = Vec::new();
        let mut chars = src.chars().peekable();
        let mut span = Span::new(1);

        /// single: if the next character doesn't match
        /// double: if the next character does match
        /// ch: next character
        macro_rules! maybe_double {
            ($single:expr, $double:expr, $ch:expr) => {{
                chars.next();
                let kind = if chars.peek() == Some(&$ch) {
                    chars.next();
                    $double
                } else {
                    $single
                };

                tokens.push(TokenContext {
                    kind,
                    span: Some(span),
                });
            }};
        }

        macro_rules! skip_whitespace {
            () => {{
                Self::skip_while(&mut chars, |c| c.is_whitespace());
            }};
        }

        while let Some(&ch) = chars.peek() {
            match ch {
                CH_WHITESPACE | CH_TB | CH_CR => {
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

                    skip_whitespace!();

                    let token = match ident.as_str() {
                        KW_LET => Token::Let,
                        KW_OUT => Token::Out,
                        KW_IN => Token::In,
                        KW_IF => Token::If,
                        KW_FOR => Token::For,
                        KW_LOOP => Token::Loop,
                        KW_WHILE => Token::While,
                        KW_BREAK => Token::Break,
                        KW_RETURN => Token::Return,
                        KW_CONTINUE => Token::Continue,
                        KW_ELSE => Token::Else,
                        KW_TRUE => Token::Boolean(true),
                        KW_FALSE => Token::Boolean(false),
                        _ => {
                            let mut sid: Option<String> = None;
                            if let Some(&':') = chars.peek() {
                                chars.next();

                                skip_whitespace!();
                                let type_name =
                                    Self::collect_while(&mut chars, |c| c.is_alphabetic());

                                if !type_name.is_empty() {
                                    sid = Some(type_name);
                                }
                            }

                            Token::Ident {
                                name: ident.to_string(),
                                sid,
                            }
                        }
                    };

                    tokens.push(TokenContext {
                        kind: token,
                        span: Some(span),
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
                        kind: Token::Integer(num),
                        span: Some(span),
                    });
                }
                CH_DOT => {
                    chars.next();

                    if chars.peek() == Some(&'.') {
                        chars.next();

                        let token = if chars.peek() == Some(&'=') {
                            chars.next();
                            Token::DotDotEqual
                        } else {
                            Token::DotDot
                        };

                        tokens.push(TokenContext {
                            kind: token,
                            span: Some(span),
                        });
                    }
                }
                CH_LCURLY | CH_RCURLY => {
                    tokens.push(TokenContext {
                        kind: if ch == CH_LCURLY {
                            Token::LCurly
                        } else {
                            Token::RCurly
                        },
                        span: Some(span),
                    });
                    chars.next();
                }
                CH_ADD | CH_SUB | CH_MUL | CH_DIV | CH_MOD | CH_SEMICOLON | CH_LPARAN
                | CH_RPARAN | CH_COMMA => {
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
                        span: Some(span),
                    });
                    chars.next();
                }
                CH_NOT => maybe_double!(Token::Bang, Token::BangEqual, CH_EQ),
                CH_EQ => maybe_double!(Token::Equal, Token::EqualEqual, CH_EQ),
                CH_GT => maybe_double!(Token::Greater, Token::GreaterEqual, CH_EQ),
                CH_LT => maybe_double!(Token::Lesser, Token::LesserEqual, CH_EQ),
                CH_OR => maybe_double!(Token::Or, Token::OrOr, CH_OR),
                CH_AND => maybe_double!(Token::And, Token::AndAnd, CH_AND),
                c => return Err(lex_err!(LexerError::UnknownCharacter(c), span)),
            }
        }

        // tokens.push(TokenContext::from(Token::EOF));
        Ok(tokens)
    }

    pub fn precedence(&self) -> Option<u8> {
        match self {
            Token::Bang => Some(20),
            Token::Star | Token::Slash | Token::Percent => Some(19),
            Token::Plus | Token::Minus => Some(18),
            Token::Lesser | Token::LesserEqual | Token::Greater | Token::GreaterEqual => Some(17),
            Token::EqualEqual | Token::BangEqual => Some(16),
            Token::And => Some(15),
            // XOR
            Token::Or => Some(13),
            Token::AndAnd => Some(12),
            Token::OrOr => Some(11),
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

            let opr_kind: OperationKind = match &op_token.kind {
                // arithmetic
                Token::Plus => OperationKind::Arithmetic(BinOp::Add),
                Token::Minus => OperationKind::Arithmetic(BinOp::Sub),
                Token::Star => OperationKind::Arithmetic(BinOp::Mul),
                Token::Slash => OperationKind::Arithmetic(BinOp::Div),
                Token::Percent => OperationKind::Arithmetic(BinOp::Mod),

                // cmp
                Token::AndAnd => OperationKind::Comparative(CmpOp::And),
                Token::OrOr => OperationKind::Comparative(CmpOp::Or),
                Token::Lesser => OperationKind::Comparative(CmpOp::Lt),
                Token::LesserEqual => OperationKind::Comparative(CmpOp::Le),
                Token::Greater => OperationKind::Comparative(CmpOp::Gt),
                Token::GreaterEqual => OperationKind::Comparative(CmpOp::Ge),
                Token::EqualEqual => OperationKind::Comparative(CmpOp::Eq),
                Token::BangEqual => OperationKind::Comparative(CmpOp::Ne),

                expr => return Err(lex_err!(LexerError::InvalidExpression(expr.to_string()))),
            };

            self.next();
            let rhs = self.parse_expression(prec + 1)?;

            lhs = match opr_kind {
                OperationKind::Arithmetic(op) => Expression::BinOp {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    op,
                },
                OperationKind::Comparative(op) => Expression::BoolOp {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    op,
                },
            };
        }

        Ok(lhs)
    }

    pub fn parse_leaf(&mut self) -> Result<Expression, CompileError> {
        if let Some(token) = self.next() {
            let span = token.span;

            match &token.kind {
                Token::Integer(n) => Ok(Expression::Value(Signal::from(*n))),
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
