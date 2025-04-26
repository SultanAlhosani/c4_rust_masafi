mod lexer;
mod parser;
mod ast;
mod vm;

use lexer::Lexer;
use parser::Parser;
use vm::Vm;

fn main() {
    let source_code = "
    let x = 0;
    let y = 10;
    if (true) { x = x + 5; }
    while (x < 10) { x = x + 1; }
    return x + y;
    ";

    // Initialize components
    let lexer = Lexer::new(source_code);
    let mut vm = Vm::new();
    let mut parser = Parser::new(lexer, &mut vm);

    // Parse source code into statements
    let statements = parser.parse();

    // Execute statements
    for stmt in statements {
        vm.execute(stmt);
    }

    // Final output
    println!("Program finished. Final result = {}", vm.get_result());
}
