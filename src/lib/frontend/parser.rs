use super::ast::*;
use super::lexemes;
use super::token::*;

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
        Ok(Self {
            cursor: 0,
            tokens: Token::tokenize(src)?,
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
            let span = *span;
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
                Token::If => self.parse_if(),
                Token::For => todo!(),
                Token::Loop => self.parse_loop(),
                Token::While => todo!(),
                Token::Break => {
                    self.consume()?;
                    self.expect(Token::Semicolon)?;
                    Ok(StatementKind::Break)
                }
                Token::Continue => todo!(),
                Token::Return => todo!(),
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

            let tokens = self.consume_until(Token::Semicolon)?;
            let expr = Expresso::new(&tokens).parse_expression(0)?;
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
                    found: token.kind.to_string()
                },
                token.span
            ))
        }
    }

    fn parse_block_literal(&mut self) -> Result<Vec<StatementContext>, CompileError> {
        self.expect(Token::LCurly)?;
        let body = self.parse_until(Token::RCurly)?;
        self.expect(Token::RCurly)?;

        Ok(body)
    }

    fn parse_block(&mut self) -> Result<StatementKind, CompileError> {
        Ok(StatementKind::Block {
            body: self.parse_block_literal()?,
        })
    }

    fn parse_if(&mut self) -> Result<StatementKind, CompileError> {
        self.expect(Token::If)?;

        let tokens = self.collect_until(Token::LCurly)?;
        let then = self.parse_block_literal()?;

        let mut alter = None;
        if let Some(Token::Else) = self.peek() {
            self.consume()?;
            alter = Some(self.parse_block_literal()?);
        }

        let expr = Expresso::new(&tokens).parse_expression(0)?;
        Ok(StatementKind::If {
            cond: expr,
            then,
            alter,
        })
    }

    fn parse_loop(&mut self) -> Result<StatementKind, CompileError> {
        self.expect(Token::Loop)?;
        let body = self.parse_block_literal()?;
        Ok(StatementKind::Loop { body })
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

    fn until(
        &mut self,
        limit: Token,
        consume_limit: bool,
    ) -> Result<Vec<TokenContext>, CompileError> {
        let mut collected = Vec::new();
        while let Some(TokenContext { kind, .. }) = self.peek_context() {
            if kind == &limit {
                if consume_limit {
                    self.consume()?;
                }
                break;
            }
            collected.push(self.consume()?);
        }
        Ok(collected)
    }

    fn consume_until(&mut self, limit: Token) -> Result<Vec<TokenContext>, CompileError> {
        self.until(limit, true)
    }

    fn collect_until(&mut self, limit: Token) -> Result<Vec<TokenContext>, CompileError> {
        self.until(limit, false)
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
