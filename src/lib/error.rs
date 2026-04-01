use colored::Colorize;

use crate::frontend::ast::*;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken { found: String },
    UnexpectedPattern,
    UnexpectedVariant,
    UnknownCharacter { found: String },
    UnknownSignalId { found: String },
    UnexpectedEof,
    UnmatchedParenthesis,
    MissingSemicolon,
    MissingSignalType,
    InvalidIdentifier,
    ReservedKeyword { keyword: String },
}

pub fn maybe_span<T: IntoOptSpan>(s: T) -> Option<Span> {
    s.into_opt_span()
}

#[macro_export]
macro_rules! parse_err {
    ($err:expr, $span:expr) => {
        CompileError::new(
            CompileErrorKind::Parse($err),
            $crate::error::maybe_span($span),
        )
    };
    ($err:expr, $span:expr) => {
        CompileError::new(CompileErrorKind::Parse($err), Some($span))
    };
    ($err:expr) => {
        CompileError::new(CompileErrorKind::Parse($err), None)
    };
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken { found } => write!(f, "unexpected token '{}'.", found),
            Self::UnexpectedPattern => write!(f, "unexpected pattern."),
            Self::UnexpectedVariant => write!(f, "unexpected variant."),
            Self::UnexpectedEof => write!(f, "unexpected end of input."),
            Self::UnknownSignalId { found } => write!(f, "unknown signal id: {}", found),
            Self::UnknownCharacter { found } => write!(f, "unknown character '{}'.", found),
            Self::UnmatchedParenthesis => write!(f, "unmatched parenthesis."),
            Self::MissingSemicolon => write!(f, "missing semicolon."),
            Self::MissingSignalType => write!(f, "missing signal type."),
            Self::InvalidIdentifier => write!(f, "invalid identifier."),
            Self::ReservedKeyword { keyword } => {
                write!(f, "expected identifier, {} is a reserved keyword.", keyword)
            }
        }
    }
}

#[derive(Debug)]
pub enum LexerError {
    UnmatchedParenthesis,
    UnknownCharacter(char),
    UnexpectedEndOfInput,
    InvalidExpression(String),
}

#[macro_export]
macro_rules! lex_err {
    ($err:expr, $span:expr) => {
        CompileError::new(
            CompileErrorKind::Lex($err),
            $crate::error::maybe_span($span),
        )
    };
    ($err:expr, $span:expr) => {
        CompileError::new(CompileErrorKind::Lex($err), Some($span))
    };
    ($err:expr) => {
        CompileError::new(CompileErrorKind::Lex($err), None)
    };
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnmatchedParenthesis => write!(f, "unmatched parenthesis."),
            Self::UnknownCharacter(c) => write!(f, "unknown character '{}'.", c),
            Self::UnexpectedEndOfInput => write!(f, "unexpected end of input."),
            Self::InvalidExpression(expr) => write!(f, "invalid expression: {}.", expr),
        }
    }
}

#[derive(Debug)]
pub enum SemanticError {
    UndefinedVariable(String),
    DuplicateVariable(String),
    TypeMismatch { expected: String, found: String },
    InvalidAssignmentTarget,
}

#[macro_export]
macro_rules! semantic_err {
    ($err:expr, $span:expr) => {
        CompileError::new(CompileErrorKind::Semantic($err), Some($span))
    };
    ($err:expr) => {
        CompileError::new(CompileErrorKind::Semantic($err), None)
    };
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndefinedVariable(name) => write!(f, "undefined variable '{}'.", name),
            Self::DuplicateVariable(name) => write!(f, "duplicate variable '{}'.", name),
            Self::TypeMismatch { expected, found } => write!(
                f,
                "type mismatch - expected '{}', found '{}'.",
                expected, found
            ),
            Self::InvalidAssignmentTarget => write!(f, "invalid assignment target."),
        }
    }
}

#[derive(Debug)]
pub enum GeneratorError {
    UndefinedVariable(String),
    OutOfRegisters,
    RegisterDoubleFree(u8),
    RegisterNotAllocated(u8),
    RegisterCannotBeTyped,
    InvalidRegister,
    NonAddressableLocation,
    NonAddressableSymbol,
    InvalidInstruction { msg: String },
}

#[macro_export]
macro_rules! gen_err {
    ($err:expr, $span:expr) => {
        CompileError::new(CompileErrorKind::Generation($err), Some($span))
    };
    ($err:expr) => {
        CompileError::new(CompileErrorKind::Generation($err), None)
    };
}

impl fmt::Display for GeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndefinedVariable(name) => write!(f, "undefined variable '{}'.", name),
            Self::OutOfRegisters => write!(f, "out of registers."),
            Self::RegisterDoubleFree(reg) => {
                write!(f, "double free of register r{}.", reg)
            }
            Self::RegisterNotAllocated(reg) => {
                write!(f, "register r{} was not allocated.", reg)
            }
            Self::RegisterCannotBeTyped => write!(f, "register cannot be typed."),
            Self::InvalidRegister => write!(f, "invalid register."),
            Self::NonAddressableLocation => write!(f, "non-addressable location."),
            Self::NonAddressableSymbol => write!(f, "non-addressable symbol."),
            Self::InvalidInstruction { msg } => write!(f, "invalid instruction: {}", msg),
        }
    }
}

#[derive(Debug)]
pub struct CompileError {
    pub kind: CompileErrorKind,
    pub span: Option<Span>,
}

impl CompileError {
    pub fn new(kind: CompileErrorKind, span: Option<Span>) -> Self {
        Self { kind, span }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use colored::Colorize;
        match &self.span {
            Some(span) => write!(
                f,
                "{} {}",
                format!("[line: {}]", span.line).dimmed(),
                self.kind
            ),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl std::error::Error for CompileError {}

#[derive(Debug)]
pub enum CompileErrorKind {
    Lex(LexerError),
    Parse(ParseError),
    Semantic(SemanticError),
    Generation(GeneratorError),
}

impl fmt::Display for CompileErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lex(err) => write!(f, "{} {}", "Lexer error:".bold().red(), err),
            Self::Parse(err) => write!(f, "{} {}", "Parser error:".bold().red(), err),
            Self::Semantic(err) => write!(f, "{} {}", "Semantic error:".bold().red(), err),
            Self::Generation(err) => write!(f, "{} {}", "Generation error:".bold().red(), err),
        }
    }
}

pub use crate::gen_err;
pub use crate::lex_err;
pub use crate::parse_err;
pub use crate::semantic_err;
