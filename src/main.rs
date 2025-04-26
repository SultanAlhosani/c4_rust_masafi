mod lexer;
mod parser;
mod vm;
mod ast;

use lexer::Lexer;
use parser::Parser;
use vm::Vm;
use ast::Stmt;

fn main() {
    let source_code = "
        return 2 + 3 * 4;
    ";

    // Initialize components
    let lexer = Lexer::new(source_code);
    let mut vm = Vm::new();
    let mut parser = Parser::new(lexer, &mut vm);

    // Parse source code into statements
    let statements: Vec<Stmt> = parser.parse();

    // Execute statements
    for stmt in statements {
        vm.execute(stmt);
    }

    // Final output
    println!("Program finished. Final result = {}", vm.get_result());
}
