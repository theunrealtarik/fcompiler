pub static KW_LET: &str = "let";
pub static KW_OUT: &str = "out";

pub static KW_IF: &str = "if";
pub static KW_ELSE: &str = "else";
pub static KW_FOR: &str = "for";
pub static KW_WHILE: &str = "while";

pub static KW_TRUE: &str = "true";
pub static KW_FALSE: &str = "false";

pub static RESERVED_KEYWORDS: [&str; 8] = [
    KW_LET, KW_OUT, KW_IF, KW_ELSE, KW_FOR, KW_WHILE, KW_TRUE, KW_FALSE,
];

pub static CH_ADD: char = '+';
pub static CH_SUB: char = '-';
pub static CH_MUL: char = '*';
pub static CH_DIV: char = '/';
pub static CH_MOD: char = '%';

pub static CH_EQ: char = '=';
pub static CH_LT: char = '<';
pub static CH_GT: char = '>';

pub static CH_NOT: char = '!';
pub static CH_AND: char = '&';
pub static CH_OR: char = '|';

pub static CH_COMMA: char = ',';
pub static CH_COLON: char = ':';
pub static CH_SEMICOLON: char = ';';
pub static CH_WHITESPACE: char = ' ';
pub static CH_UNDERSCORE: char = '_';

pub static CH_LPARAN: char = '(';
pub static CH_RPARAN: char = ')';

pub static OPERATOR_CHARS: [char; 11] = [
    CH_ADD, CH_SUB, CH_MUL, CH_DIV, CH_MOD, CH_EQ, CH_LT, CH_GT, CH_NOT, CH_AND, CH_OR,
];
