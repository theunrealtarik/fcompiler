pub const KW_LET: &str = "let";
pub const KW_OUT: &str = "out";

pub const KW_IF: &str = "if";
pub const KW_ELSE: &str = "else";
pub const KW_FOR: &str = "for";
pub const KW_WHILE: &str = "while";

pub const KW_TRUE: &str = "true";
pub const KW_FALSE: &str = "false";

pub const RESERVED_KEYWORDS: [&str; 8] = [
    KW_LET, KW_OUT, KW_IF, KW_ELSE, KW_FOR, KW_WHILE, KW_TRUE, KW_FALSE,
];

pub const CH_ADD: char = '+';
pub const CH_SUB: char = '-';
pub const CH_MUL: char = '*';
pub const CH_DIV: char = '/';
pub const CH_MOD: char = '%';

pub const CH_EQ: char = '=';
pub const CH_LT: char = '<';
pub const CH_GT: char = '>';

pub const CH_NOT: char = '!';
pub const CH_AND: char = '&';
pub const CH_OR: char = '|';

pub const CH_COMMA: char = ',';
pub const CH_COLON: char = ':';
pub const CH_SEMICOLON: char = ';';
pub const CH_WHITESPACE: char = ' ';
pub const CH_UNDERSCORE: char = '_';
pub const CH_TAB: char = '\t';
pub const CH_NL: char = '\n';

pub const CH_LPARAN: char = '(';
pub const CH_RPARAN: char = ')';
pub const CH_LCURLY: char = '{';
pub const CH_RCURLY: char = '}';

pub const OPERATOR_CHARS: [char; 11] = [
    CH_ADD, CH_SUB, CH_MUL, CH_DIV, CH_MOD, CH_EQ, CH_LT, CH_GT, CH_NOT, CH_AND, CH_OR,
];
