#[derive(Debug)]
pub struct AssemblyFormatter;

impl AssemblyFormatter {
    pub fn mov<D: std::fmt::Display, S: std::fmt::Display>(dst: D, src: S) -> String {
        format!("mov {} {}\n", dst, src)
    }

    pub fn clr<D: std::fmt::Display>(dst: Option<D>) -> String {
        match dst {
            Some(d) => format!("clr {}\n", d),
            None => "clr\n".to_string(),
        }
    }

    pub fn arith<D, L, R>(op: &str, dst: D, lhs: L, rhs: R) -> String
    where
        D: std::fmt::Display,
        L: std::fmt::Display,
        R: std::fmt::Display,
    {
        format!("{} {} {} {}\n", op, dst, lhs, rhs)
    }

    pub fn inc<D: std::fmt::Display>(dst: D) -> String {
        format!("inc {}\n", dst)
    }

    pub fn dec<D: std::fmt::Display>(dst: D) -> String {
        format!("dec {}\n", dst)
    }

    pub fn neg<D>(dst: D) -> String
    where
        D: std::fmt::Display + std::fmt::Debug,
    {
        Self::arith("mul", dst, -1, "")
    }

    pub fn not<D, S>(dst: D, src: Option<S>) -> String
    where
        D: std::fmt::Display + std::fmt::Debug,
        S: std::fmt::Display + std::fmt::Debug,
    {
        format!(
            "not {} {}\n",
            dst,
            match src {
                Some(s) => format!("{}", s),
                None => String::new(),
            }
        )
    }
}
