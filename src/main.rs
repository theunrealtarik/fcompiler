use clap::Parser as ClipParser;
use std::fs::File;
use std::io::Read;

use lib::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    lib::utils::env();

    let args = Args::parse();

    let mut file = File::open(args.file)?;
    let mut src = String::new();
    file.read_to_string(&mut src)?;

    let mut assembler = Assembler::new(Parser::parse(&src).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    }));

    match assembler.finish() {
        Ok(code) => {
            println!("{}", code);
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
    Ok(())
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: String,
}
