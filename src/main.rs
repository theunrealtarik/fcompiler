use lib::backend::codegen::Generator;
use lib::frontend::parser;

fn main() {
    let source = r#"
        let x = 2;
        let y = 3;

        let z = -x + y;
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
