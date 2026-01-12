use lib::backend::codegen::Generator;
use lib::frontend::parser;

fn main() {
    let source = r#"
    let r = 2 * 3 + 1;
    let x = 2 * 5 * 3;
    let h = r + x;
    "#;

    let mut generator = Generator::new(parser::parse(source).unwrap());
    let code = generator.generate().unwrap();
    print!("\n{}", code);
}
