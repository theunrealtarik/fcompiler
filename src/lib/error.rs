#[derive(Debug, strum_macros::Display)]
pub enum CompileError {
    // Parser errors
    MissingSemicolon(String),
    UnexpectedPattern(String),
    InvalidIdentifier(String),
    UnexpectedToken(String),
    UnmatchedParenthesis,

    // Expression / Tokenizer errors
    UnknownCharacter(char),
    UnexpectedEndOfInput,
    InvalidExpression(String),

    // Codegen errors
    UndefinedVariable(String),
    OutOfRegisters,
}
