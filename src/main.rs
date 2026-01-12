use lib::backend::codegen::Generator;
use lib::frontend::parser;

fn main() {
    let source = r#"
    let x = 10 * (5 + 2);
    "#;

    let mut generator = Generator::new(parser::parse(source).unwrap());
    let code = generator.generate().unwrap();
    print!("\n{}", code);
}
