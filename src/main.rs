use lib::backend::codegen::Generator;
use lib::frontend::parser;

fn main() {
    let source = r#"
        let x = 5;
        let y = x * 2;

        let a: IronPlate = 10;
        let b = (a * 5) / (x - y);

        out(a);
    "#;

    let mut generator = Generator::new(parser::parse(source).unwrap());
    let code = generator.generate().unwrap();
    print!("\n{}", code);
}
