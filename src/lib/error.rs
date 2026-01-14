use crate::frontend::ast::Span;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken { found: String },
    UnexpectedPattern,
    UnexpectedVariant,
    UnexpectedEof,
    UnmatchedParenthesis,
    MissingSemicolon,
    MissingSignalType,
    InvalidIdentifier,
    ReservedKeyword { keyword: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { found } => write!(f, "unexpected token '{}'.", found),
            ParseError::UnexpectedPattern => write!(f, "unexpected pattern."),
            ParseError::UnexpectedVariant => write!(f, "unexpected variant."),
            ParseError::UnexpectedEof => write!(f, "unexpected end of input."),
            ParseError::UnmatchedParenthesis => write!(f, "unmatched parenthesis."),
            ParseError::MissingSemicolon => write!(f, "missing semicolon."),
            ParseError::MissingSignalType => write!(f, "missing signal type."),
            ParseError::InvalidIdentifier => write!(f, "invalid identifier."),
            ParseError::ReservedKeyword { keyword } => {
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

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerError::UnmatchedParenthesis => write!(f, "unmatched parenthesis."),
            LexerError::UnknownCharacter(c) => write!(f, "unknown character '{}'.", c),
            LexerError::UnexpectedEndOfInput => write!(f, "unexpected end of input."),
            LexerError::InvalidExpression(expr) => write!(f, "invalid expression: {}.", expr),
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

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticError::UndefinedVariable(name) => write!(f, "undefined variable '{}'.", name),
            SemanticError::DuplicateVariable(name) => write!(f, "duplicate variable '{}'.", name),
            SemanticError::TypeMismatch { expected, found } => write!(
                f,
                "type mismatch - expected '{}', found '{}'.",
                expected, found
            ),
            SemanticError::InvalidAssignmentTarget => write!(f, "invalid assignment target."),
        }
    }
}

#[derive(Debug)]
pub enum GeneratorError {
    UndefinedVariable(String),
    OutOfRegisters,
    RegisterDoubleFree(u8),
    RegisterNotAllocated(u8),
    InvalidRegister,
    NonAddressableLocation,
}

impl fmt::Display for GeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeneratorError::UndefinedVariable(name) => write!(f, "undefined variable '{}'.", name),
            GeneratorError::OutOfRegisters => write!(f, "out of registers."),
            GeneratorError::RegisterDoubleFree(reg) => {
                write!(f, "double free of register r{}.", reg)
            }
            GeneratorError::RegisterNotAllocated(reg) => {
                write!(f, "register r{} was not allocated.", reg)
            }
            GeneratorError::InvalidRegister => write!(f, "invalid register."),
            GeneratorError::NonAddressableLocation => write!(f, "non-addressable location."),
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
        match &self.span {
            Some(span) => write!(f, "(line: {}) {}", span.line, self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

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
            CompileErrorKind::Lex(err) => write!(f, "Lexer error: {}", err),
            CompileErrorKind::Parse(err) => write!(f, "Parser error: {}", err),
            CompileErrorKind::Semantic(err) => write!(f, "Semantics error: {}", err),
            CompileErrorKind::Generation(err) => write!(f, "Generation error: {}", err),
        }
    }
}

impl std::error::Error for CompileError {}
