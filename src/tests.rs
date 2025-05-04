use crate::{lexer::Lexer, parser::Parser, vm::{Vm, Value}};

fn run(code: &str) -> i32 {
    let lexer = Lexer::new(code);
    let mut vm = Vm::new();
    let mut parser = Parser::new(lexer, &mut vm);
    let stmts = parser.parse();
    for stmt in stmts {
        vm.execute(stmt);
    }
    vm.get_result()
}

#[test]
fn test_array_element() {
    let code = "
        let arr = [100, 200, 300];
        return arr[1];
    ";
    assert_eq!(run(code), 200);
}
