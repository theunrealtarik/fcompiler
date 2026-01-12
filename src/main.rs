use lib::backend::codegen::Generator;
use lib::frontend::parser;

fn main() {
    let source = r#"
    let n = 2;
    let m = 3;
    let r = n * m + 1;
    out(r);
    "#;

    let mut generator = Generator::new(parser::parse(source).unwrap());
    let code = generator.generate().unwrap();
    print!("\n{}", code);
}
