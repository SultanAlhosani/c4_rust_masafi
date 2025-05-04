/// This program reads a C4 source file, tokenizes it using the lexer, parses it into
/// an abstract syntax tree (AST) using the parser, and executes the resulting statements
/// using the virtual machine (VM).
mod ast;
mod lexer;
mod parser;
mod vm;

use lexer::Lexer;
use parser::Parser;
use std::fs;
use vm::Vm;


fn main() {
    // Read the source code from the C4 file.
    let source_code =
        fs::read_to_string("examples/compiler.c4").expect("Failed to read C4 source file");

    // Initialize the lexer, parser, and virtual machine.
    let lexer = Lexer::new(&source_code);
    let mut vm = Vm::new();
    let mut parser = Parser::new(lexer, &mut vm);

    // Parse the source code into a list of statements.
    let statements = parser.parse();

    // Execute each statement using the virtual machine.
    for stmt in statements {
        vm.execute(stmt);
    }

    // Print the final result of the program.
    if let Some(s) = vm.get_result_str() {
        println!("Program finished. Final result = \"{}\"", s);
    } else {
        println!("Program finished. Final result = {}", vm.get_result());
    }
    
}
