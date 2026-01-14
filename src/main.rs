use clap::Parser;
use lib::backend::codegen::Generator;
use lib::frontend::parser;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    lib::utils::env();

    let args = Args::parse();

    let mut file = File::open(args.file)?;
    let mut src = String::new();
    file.read_to_string(&mut src)?;

    let mut generator = Generator::new(parser::parse(&src).unwrap_or_else(|err| {
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
    Ok(())
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: String,
}
