pub mod backend;
pub mod frontend;

pub mod error;
pub mod game;

pub mod utils {
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
