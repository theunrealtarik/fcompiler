use lib::backend::codegen::Generator;
use lib::frontend::parser;

fn main() {
    lib::utils::env();

    let source = r#"
        let x = 5;
        let y = -(x + 2);

        let z = y + 2;
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
