use super::mem::*;

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

    // Arithematics
    fn arith_op<O, D, S, V>(&mut self, op: &O, dst: D, src: Option<S>, val: V)
    where
        O: std::fmt::Display,
        D: std::fmt::Display,
        S: std::fmt::Display,
        V: std::fmt::Display,
    {
        let rhs = match src {
            Some(s) => format!("{} {}", s, val),
            None => format!("{}", val),
        };

        let code = format!("{} {} {}\n", op, dst, rhs);
        self.code.push_str(&code);
    }

    // ADD
    /// add dst src? val
    pub fn add<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display,
        S: std::fmt::Display,
        V: std::fmt::Display,
    {
        self.arith_op(&"add", dst, src, val);
    }

    /// dst = dst + val
    pub fn addi<D, V>(&mut self, dst: D, val: V)
    where
        D: std::fmt::Display,
        V: std::fmt::Display,
    {
        self.add::<D, String, V>(dst, None, val);
    }

    /// dst = dst + src
    pub fn add_r<D, S>(&mut self, dst: D, src: S)
    where
        D: std::fmt::Display,
        S: std::fmt::Display,
    {
        self.add(dst, Some(src), "");
    }

    // SUBTRACT
    pub fn sub<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display,
        S: std::fmt::Display,
        V: std::fmt::Display,
    {
        self.arith_op(&"sub", dst, src, val);
    }

    pub fn sub_r<D, S>(&mut self, dst: D, src: S)
    where
        D: std::fmt::Display,
        S: std::fmt::Display,
    {
        self.sub(dst, Some(src), "");
    }

    // MULTIPLY
    pub fn mul<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display,
        S: std::fmt::Display,
        V: std::fmt::Display,
    {
        self.arith_op(&"mul", dst, src, val);
    }

    pub fn mul_r<D, S>(&mut self, dst: D, src: S)
    where
        D: std::fmt::Display,
        S: std::fmt::Display,
    {
        self.mul(dst, Some(src), "");
    }

    // DIVIDE
    pub fn div<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display,
        S: std::fmt::Display,
        V: std::fmt::Display,
    {
        self.arith_op(&"div", dst, src, val);
    }

    pub fn div_r<D, S>(&mut self, dst: D, src: S)
    where
        D: std::fmt::Display,
        S: std::fmt::Display,
    {
        self.div(dst, Some(src), "");
    }

    // MODULO (remainder)
    pub fn modu<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display,
        S: std::fmt::Display,
        V: std::fmt::Display,
    {
        self.arith_op(&"mod", dst, src, val);
    }

    pub fn modu_r<D, S>(&mut self, dst: D, src: S)
    where
        D: std::fmt::Display,
        S: std::fmt::Display,
    {
        self.modu(dst, Some(src), "");
    }

    pub fn finish(&self) -> &str {
        &self.code
    }
}
