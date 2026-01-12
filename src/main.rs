use lib::backend::codegen::Generator;
use lib::frontend::parser;

fn main() {
    let source = r#"
    let test = 5 * 2 / (1 + 3) * 4;
    out(test);
    "#;

    let mut generator = Generator::new(parser::parse(source).unwrap());
    let code = generator.generate().unwrap();
    print!("\n{}", code);
}
