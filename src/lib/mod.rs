pub mod backend;
pub mod frontend;

pub mod error;
pub mod game;

pub mod compiler {
    use crate::frontend::ast::Program;
    use crate::frontend::parser::Parser;
    use crate::log;

    pub struct Compiler;

    impl Compiler {
        pub fn compile(src: &str) -> Result<(), crate::error::CompileError> {
            let stmts = Parser::new(src)?.parse()?;
            let program = Program::from(stmts);

            log::debug!("{:#?}", program);
            // Assembler::new(program);
            Ok(())
        }
    }
}

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

pub mod log {
    use colored::Colorize;
    use tracing_subscriber::fmt::{FormatEvent, FormatFields};

    pub fn init() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .event_format(CompilerLogFormatter)
            .init();
    }

    pub use tracing::debug;
    pub use tracing::error;
    pub use tracing::info;
    pub use tracing::warn;

    #[macro_export]
    macro_rules! asm {
        ($($arg:tt)*) => {
            tracing::debug!(target: "asm", $($arg)*)
        };
    }

    pub use crate::asm;

    #[allow(dead_code)]
    struct CompilerLogFormatter;

    impl<S, N> FormatEvent<S, N> for CompilerLogFormatter
    where
        S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
        N: for<'a> tracing_subscriber::fmt::FormatFields<'a> + 'static,
    {
        fn format_event(
            &self,
            ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
            mut writer: tracing_subscriber::fmt::format::Writer<'_>,
            event: &tracing::Event<'_>,
        ) -> std::fmt::Result {
            let meta = event.metadata();

            let level = match *meta.level() {
                tracing::Level::ERROR => "ERROR".bold().red(),
                tracing::Level::WARN => "WARN ".yellow().bold(),
                tracing::Level::INFO => "INFO ".green().bold(),
                tracing::Level::DEBUG => "DEBUG".blue().bold(),
                tracing::Level::TRACE => "TRACE".green().bold(),
            };

            if meta.target() == "asm" {
                write!(
                    writer,
                    "{}{}{} ",
                    "[".dimmed(),
                    " ASM ".purple().bold(),
                    "]".dimmed(),
                )?;
            } else {
                write!(
                    writer,
                    "{}{}{} {} ",
                    "[".dimmed(),
                    level,
                    "]".dimmed(),
                    meta.target().dimmed()
                )?;
            }

            ctx.format_fields(writer.by_ref(), event)?;
            writeln!(writer)
        }
    }
}
