use lib::codegen::Generator;
use lib::parser;

fn main() {
    let source = r#"
    let x = 10;
    let y = x + 5;
    "#;

    let mut generator = Generator::new(parser::parse(source).unwrap());
    let code = generator.generate().unwrap();
    print!("\n{}", code);
}
