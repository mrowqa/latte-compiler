extern crate latte_compiler;

use latte_compiler::compile;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<_> = env::args().collect();

    if !(args.len() == 2 || (args.len() == 3 && args[1] == "--make-executable")) {
        eprintln!("Usage: {} [--make-executable] <filename.lat>", args[0]);
        process::exit(1);
    }
    let make_executable = args.len() == 3;

    let input_file_str = &args[args.len() - 1];
    let input_file = Path::new(&input_file_str);
    let code = match fs::read_to_string(input_file) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Cannot read file: {}", input_file.display());
            process::exit(1);
        }
    };

    let res = compile(input_file_str, &code);
    let ll_code = match res {
        Ok(prog) => {
            eprintln!("OK");
            format!("{}", prog)
        }
        Err(msg) => {
            eprintln!("ERROR");
            eprintln!("{}", msg);
            process::exit(1);
        }
    };

    let ll_output_file = input_file.with_extension("ll");
    let bc_output_file = input_file.with_extension("bc");
    match fs::write(&ll_output_file, ll_code) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("Cannot write file: {}", ll_output_file.display());
            process::exit(1);
        }
    }

    if run_command(&[
        "llvm-as",
        "-o",
        bc_output_file.to_str().unwrap(),
        ll_output_file.to_str().unwrap(),
    ]) {
        println!(
            "Compiled {} to {} and {}.",
            input_file.display(),
            ll_output_file.display(),
            bc_output_file.display()
        );
    } else {
        eprintln!("Failed to run llvm-as");
        process::exit(1);
    }

    if make_executable {
        let o_output_file = input_file.with_extension("o");
        let exec_output_file = input_file.with_extension("");
        let bc_runtime = Path::new("lib/runtime.bc");
        let o_runtime = bc_runtime.with_extension("o");

        if !Path::exists(&o_runtime) {
            println!("Compiling runtime.");
            if !run_command(&[
                "llc",
                "-march=x86",
                "-filetype=obj",
                "-o",
                o_runtime.to_str().unwrap(),
                bc_runtime.to_str().unwrap(),
            ]) {
                eprintln!(
                    "Failed to compile runtime!\nRuntime file: {}",
                    bc_runtime.display()
                );
                process::exit(1);
            }
        }

        if !run_command(&[
            "llc",
            "-march=x86",
            "-filetype=obj",
            "-o",
            o_output_file.to_str().unwrap(),
            bc_output_file.to_str().unwrap(),
        ]) {
            eprintln!("Failed to compile generated llvm bitcode.");
            process::exit(1);
        }

        if run_command(&[
            "gcc",
            "-m32",
            "-lreadline",
            "-o",
            exec_output_file.to_str().unwrap(),
            o_output_file.to_str().unwrap(),
            o_runtime.to_str().unwrap(),
        ]) {
            println!("Created executable {}", exec_output_file.display());
        } else {
            eprintln!(
                "Failed to link {} and {} with gcc.",
                o_output_file.display(),
                o_runtime.display()
            );
            process::exit(1);
        }
    }
}

fn run_command(cmd: &[&str]) -> bool {
    let result = process::Command::new(cmd[0]).args(&cmd[1..]).status();
    match result {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}
