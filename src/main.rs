mod chunk;
mod compiler;
mod scanner;
mod string;
mod value;
mod vm;

use vm::*;

fn repl() {
    use std::io::{self, BufRead, Write};

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        let mut vm = VM::new();
        print!("> ");
        io::stdout().flush().expect("Couldn't flush stdout");
        match lines.next() {
            Some(Ok(line)) => vm.interpret(&line),
            _ => break,
        };
    }
}

fn run_file(path: &String) {
    use std::fs;

    let mut vm = VM::new();
    let source = fs::read_to_string(path).expect("Failed to read filed");
    let temp = &source;
    let result = vm.interpret(temp);

    if result == InterpretResult::CompileError {
        std::process::exit(65);
    }
    if result == InterpretResult::RuntimeError {
        std::process::exit(70);
    }
}

fn main() {
    use std::env;

    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => eprintln!("Usage: rustlox [path]"),
    }
}
