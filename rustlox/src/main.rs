mod chunk;
mod compiler;
mod scanner;
mod string;
mod value;
mod vm;

use vm::*;

fn repl(vm: &mut VM) {
    use std::io::{self, BufRead, Write};

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        print!("> ");
        io::stdout().flush().expect("Couldn't flush stdout");
        let result = match lines.next() {
            Some(Ok(line)) => vm.interpret(&line),
            _ => break,
        };

        if let Err(InterpretError::InternalError(message)) = result {
            eprintln!("{}", message);
        }
    }
}

fn run_file(vm: &mut VM, path: &String) {
    use std::fs;

    let source = fs::read_to_string(path).expect("Failed to read filed");
    let temp = &source;

    match vm.interpret(temp) {
        Err(InterpretError::CompileError) => std::process::exit(65),
        Err(InterpretError::RuntimeError) => std::process::exit(70),
        Err(InterpretError::InternalError(message)) => {
            eprintln!("Fatal error: {}", message);
            std::process::exit(1)
        }
        Ok(()) => (),
    }
}

fn main() {
    use std::env;

    let mut vm = VM::new();
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => repl(&mut vm),
        2 => run_file(&mut vm, &args[1]),
        _ => eprintln!("Usage: rustlox [path]"),
    }
}
