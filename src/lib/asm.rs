use crate::mem::{Out, Register};

#[derive(Debug)]
pub struct Asm {
    code: String,
}

impl Default for Asm {
    fn default() -> Self {
        Self::new()
    }
}

impl Asm {
    pub fn new() -> Self {
        Self {
            code: String::from("clr\n"),
        }
    }

    pub fn mov<D, S>(&mut self, dst: D, src: S)
    where
        S: std::fmt::Display,
        D: std::fmt::Display,
    {
        #[allow(clippy::to_string_in_format_args)]
        self.code.push_str(&format!("mov {} {}\n", dst, src));
    }

    pub fn clr<T>(&mut self, dst: Option<T>)
    where
        T: std::fmt::Display,
    {
        let dst = match dst {
            Some(d) => format!(" {}\n", d),
            None => String::default(),
        };

        self.code.push_str(&format!("clr{}", dst));
    }

    // Move item to a CPU register
    pub fn reg_item(&mut self, reg: Register, val: &i32, sig: &Option<String>) {
        let item = format!("{}{}", val, sig.as_ref().unwrap_or(&String::new()));
        self.mov(reg, item);
    }

    // Move item to an outputting operand
    pub fn out_item(&mut self, out: Out, val: &i32, sig: &Option<String>) {
        let item = format!("{}{}", val, sig.as_ref().unwrap_or(&String::new()));
        self.mov(out, item);
    }

    // Move register to an outputting operand
    pub fn out_reg(&mut self, out: Out, reg: Register) {
        self.mov(out, reg);
    }

    pub fn add(&mut self, lhs: Register, rhs: Register) {
        self.mov_op("add", lhs, rhs);
    }

    pub fn sub(&mut self, lhs: Register, rhs: Register) {
        self.mov_op("sub", lhs, rhs);
    }

    pub fn mul(&mut self, lhs: Register, rhs: Register) {
        self.mov_op("mul", lhs, rhs);
    }

    pub fn div(&mut self, lhs: Register, rhs: Register) {
        self.mov_op("div", lhs, rhs);
    }

    pub fn modu(&mut self, lhs: Register, rhs: Register) {
        self.mov_op("mod", lhs, rhs);
    }

    fn mov_op(&mut self, op: &str, lhs: Register, rhs: Register) {
        #[allow(clippy::to_string_in_format_args)]
        self.code.push_str(&format!("{} {} {}\n", op, lhs, rhs));
    }

    pub fn finish(&self) -> &str {
        &self.code
    }
}
