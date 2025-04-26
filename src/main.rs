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
    while (1) return 5;
";

    let lexer = Lexer::new(source_code);
    let mut vm = Vm::new();
    let mut parser = Parser::new(lexer, &mut vm);

    let statements = parser.parse();

    for stmt in statements {
        vm.execute(stmt);
    }

    println!("Program finished. Final result = {}", vm.get_result());
}

