use lib::backend::codegen::Generator;
use lib::frontend::parser;

fn main() {
    let source = r#"
        let x = 1;
        x = x + 1;

        let m = -x / 2;
        out(m);
    "#;

    let mut generator = Generator::new(parser::parse(source).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    }));

    match generator.generate() {
        Ok(code) => println!("{}", code),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
