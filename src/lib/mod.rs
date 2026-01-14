pub mod backend;
pub mod frontend;

pub mod error;
pub mod game;

pub mod utils {
    pub fn env() {
        env_logger::builder().format_timestamp(None).init();
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

pub mod cli {}
