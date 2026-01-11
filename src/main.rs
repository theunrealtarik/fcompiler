use lib::codegen::Generator;
use lib::parser;

fn main() {
    let source = r#"
    let y: CopperPlate = 2;
    let x: IronPlate = 5 + 1 * y;
    out(x);
    "#;

    let mut generator = Generator::new(parser::parse(source).unwrap());
    let code = generator.generate().unwrap();
    print!("\n{}", code);
}
