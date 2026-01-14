use super::mem::*;
use log::debug;

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
        S: std::fmt::Display + std::fmt::Debug,
        D: std::fmt::Display + std::fmt::Debug,
    {
        debug!("mov {:?} <- {:?}", dst, src);
        #[allow(clippy::to_string_in_format_args)]
        self.code.push_str(&format!("mov {} {}\n", dst, src));
    }

    pub fn clr<T>(&mut self, dst: Option<T>)
    where
        T: std::fmt::Display + std::fmt::Debug,
    {
        debug!("clr {:?}", dst.as_ref().map(|d| format!("{}", d)));

        let dst = match dst {
            Some(d) => format!(" {}\n", d),
            None => String::default(),
        };

        self.code.push_str(&format!("clr{}", dst));
    }

    // Move item to a CPU register
    pub fn reg_item(&mut self, reg: Register, val: &i32, sig: &Option<String>) {
        debug!("load immediate {}{:?} into {:?}", val, sig, reg);
        let item = format!("{}{}", val, sig.as_ref().unwrap_or(&String::new()));
        self.mov(reg, item);
    }

    // Move item to an outputting operand
    pub fn out_item(&mut self, out: Out, val: &i32, sig: &Option<String>) {
        debug!("write immediate {}{:?} to {:?}", val, sig, out);
        let item = format!("{}{}", val, sig.as_ref().unwrap_or(&String::new()));
        self.mov(out, item);
    }

    // Move register to an outputting operand
    pub fn out_reg(&mut self, out: Out, reg: Register) {
        debug!("write register {:?} to {:?}", reg, out);
        self.mov(out, reg);
    }

    // Arithmetic helper
    fn arith_op<O, D, S, V>(&mut self, op: &O, dst: D, src: Option<S>, val: V)
    where
        O: std::fmt::Display + std::fmt::Debug,
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        let rhs = match src {
            Some(s) => format!("{} {}", s, val),
            None => format!("{}", val),
        };

        let code = format!("{} {} {}\n", op, dst, rhs);
        self.code.push_str(&code);
    }

    // ADD
    pub fn add<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        match &src {
            Some(s) => debug!("{:?} = {:?} + {:?}", dst, s, val),
            None => debug!("{:?} = {:?} + {:?}", dst, dst, val),
        }
        self.arith_op(&"add", dst, src, val);
    }

    // SUB
    pub fn sub<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        match &src {
            Some(s) => debug!("{:?} = {:?} - {:?}", dst, s, val),
            None => debug!("{:?} = {:?} - {:?}", dst, dst, val),
        }
        self.arith_op(&"sub", dst, src, val);
    }

    // MUL
    pub fn mul<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        match &src {
            Some(s) => debug!("{:?} = {:?} × {:?}", dst, s, val),
            None => debug!("{:?} = {:?} × {:?}", dst, dst, val),
        }
        self.arith_op(&"mul", dst, src, val);
    }

    // DIV
    pub fn div<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        match &src {
            Some(s) => debug!("{:?} = {:?} ÷ {:?}", dst, s, val),
            None => debug!("{:?} = {:?} ÷ {:?}", dst, dst, val),
        }
        self.arith_op(&"div", dst, src, val);
    }

    // MOD
    pub fn modu<D, S, V>(&mut self, dst: D, src: Option<S>, val: V)
    where
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
        V: std::fmt::Display + std::fmt::Debug,
    {
        match &src {
            Some(s) => debug!("{:?} = {:?} % {:?}", dst, s, val),
            None => debug!("{:?} = {:?} % {:?}", dst, dst, val),
        }
        self.arith_op(&"mod", dst, src, val);
    }
    pub fn inc<D: std::fmt::Display + std::fmt::Debug>(&mut self, dst: D) {
        debug!("{:?} += 1", dst);
        self.code.push_str(&format!("inc {}\n", dst));
    }

    pub fn dec<D: std::fmt::Display + std::fmt::Debug>(&mut self, dst: D) {
        debug!("{:?} -= 1", dst);
        self.code.push_str(&format!("inc {}\n", dst));
    }

    pub fn finish(&self) -> &str {
        debug!("ASM finalized ({} bytes)", self.code.len());
        &self.code
    }
}
