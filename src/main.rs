mod ast;
mod lexer;
mod parser;
mod vm;

use lexer::Lexer;
use parser::Parser;
use vm::Vm;

fn main() {
    let source_code = "
    fn add(x, y) {
        return x + y;
    }
    let result = add(5, 10);
    return result;
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
