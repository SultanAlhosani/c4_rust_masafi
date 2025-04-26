mod lexer;
mod parser;
mod ast;
mod vm;

use lexer::Lexer;
use parser::Parser;
use vm::Vm;

fn main() {
    let source_code = "
        let x = 3;
        let y = 0;

        if (x > 2) {
            y = y + 5;
            x = x + 1;
        } else {
            y = y + 10;
        }

        while (x < 8) {
            x = x + 1;
            y = y + 2;
        }

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
