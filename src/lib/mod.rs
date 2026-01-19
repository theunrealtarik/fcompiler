pub mod backend;
pub mod frontend;

pub mod error;
pub mod game;

pub mod prelude {
    pub use crate::backend::asm::Assembler;
    pub use crate::frontend::parser::Parser;
}

pub mod utils {
    pub fn env() {
        env_logger::builder()
            .format_target(false)
            .format_timestamp(None)
            .init();
    }

    #[macro_export]
    macro_rules! chstring {
        ( $( $c:expr ),* $(,)? ) => {{
            let mut s = String::new();
            $(
                s.push_str(&std::string::ToString::to_string(&$c));
            )*
            s
        }};
    }
}
