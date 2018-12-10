extern crate latte_compiler;

use latte_compiler::compile;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <filename.lat>", args[0]);
        process::exit(1);
    }

    let input_file_str = &args[1];
    let input_file = Path::new(&input_file_str);
    let code = match fs::read_to_string(input_file) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Cannot read file: {}", input_file.display());
            process::exit(1);
        }
    };

    let res = compile(input_file_str, &code);
    match res {
        Ok(prog) => println!("{:?}", prog),
        Err(msg) => {
            println!("{}", msg);
            process::exit(1);
        }
    }
}
