use clap::Parser as ClipParser;
use std::fs::File;
use std::io::Read;

use lib::compiler::*;
use lib::log;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    lib::log::init();

    let args = Args::parse();

    let mut file = File::open(args.file)?;
    let mut src = String::new();
    file.read_to_string(&mut src)?;

    Compiler::compile(&src)?;

    Ok(())
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: String,
}
