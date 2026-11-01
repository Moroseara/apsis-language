use apsis_lexer::Lexer;
use apsis_parser::Parser;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Apsis Core Compiler (apsic) v0.1.0");
        println!("Usage: apsic <file.apsi> [-o output] [-I include_dir]");
        return;
    }

    let input_path = &args[1];
    println!(">>> Compiling Apsis source: {}", input_path);

    let source = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", input_path, e);
            process::exit(1);
        }
    };

    // 1. Lexical Analysis
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            process::exit(1);
        }
    };

    println!("✔ Tokenization successful: {} tokens generated", tokens.len());

    // 2. Syntactic Analysis (Parsing)
    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parser error: {}", e);
            process::exit(1);
        }
    };

    println!("✔ Parsing successful: {} functions parsed", program.functions.len());
    for f in &program.functions {
        println!("   - fn {} (params: {})", f.name, f.params.len());
    }

    println!(">>> Compilation verified successfully (Stage 0 Core).");
}
