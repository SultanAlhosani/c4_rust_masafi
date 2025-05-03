use crate::ast::{BinOp, Expr, Stmt, UnOp};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Stmt,
}

pub struct Vm {
    pub last_result: i32,
    pub variables: HashMap<String, i32>,
    pub functions: HashMap<String, Function>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            last_result: 0,
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn set_result(&mut self, value: i32) {
        self.last_result = value;
    }

    pub fn get_result(&self) -> i32 {
        self.last_result
    }

    pub fn execute(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Return(expr) => {
                let value = self.eval_expr(expr);
                self.set_result(value);
            }
            Stmt::Let { name, value } => {
                let val = self.eval_expr(value);
                self.variables.insert(name, val);
            }
            Stmt::Assign { name, value } => {
                let val = self.eval_expr(value);
                self.variables.insert(name, val);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_value = self.eval_expr(condition);
                if cond_value != 0 {
                    self.execute(*then_branch);
                } else if let Some(else_stmt) = else_branch {
                    self.execute(*else_stmt);
                }
            }
            Stmt::While { condition, body } => {
                while self.eval_expr(condition.clone()) != 0 {
                    self.execute(*body.clone());
                }
            }
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.execute(stmt);
                }
            }
            Stmt::Function { name, params, body } => {
                let function = Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: *body,
                };
                self.functions.insert(name, function);
            }
            Stmt::Print(expr) => {
                let value = self.eval_expr(expr);
                println!("{}", value);
            }
        }
    }

    fn eval_expr(&mut self, expr: Expr) -> i32 {
        match expr {
            Expr::Number(n) => n,
            Expr::Boolean(b) => if b { 1 } else { 0 },
            Expr::Char(c) => c as i32,
            Expr::Variable(name) => *self
                .variables
                .get(&name)
                .expect(&format!("Variable '{}' not found", name)),
            Expr::BinaryOp { op, left, right } => {
                match op {
                    BinOp::Assign => self.handle_assign(*left, *right),
                    _ => {
                        let l = self.eval_expr(*left);
                        let r = self.eval_expr(*right);
                        match op {
                            BinOp::Add => l + r,
                            BinOp::Sub => l - r,
                            BinOp::Mul => l * r,
                            BinOp::Div => {
                                if r == 0 {
                                    panic!("Division by zero");
                                }
                                l / r
                            }
                            BinOp::Equal => if l == r { 1 } else { 0 },
                            BinOp::NotEqual => if l != r { 1 } else { 0 },
                            BinOp::LessThan => if l < r { 1 } else { 0 },
                            BinOp::GreaterThan => if l > r { 1 } else { 0 },
                            BinOp::LessEqual => if l <= r { 1 } else { 0 },
                            BinOp::GreaterEqual => if l >= r { 1 } else { 0 },
                            BinOp::And => if l != 0 && r != 0 { 1 } else { 0 },
                            BinOp::Or => if l != 0 || r != 0 { 1 } else { 0 },
                            _ => unreachable!(),
                        }
                    }
                }
            }
            Expr::UnaryOp { op, expr } => {
                let val = self.eval_expr(*expr);
                match op {
                    UnOp::Not => if val == 0 { 1 } else { 0 },
                }
            }
            Expr::FunctionCall { name, args } => {
                let function = self
                    .functions
                    .get(&name)
                    .expect(&format!("Function '{}' not found", name))
                    .clone();

                let arg_values: Vec<i32> =
                    args.into_iter().map(|arg| self.eval_expr(arg)).collect();

                if arg_values.len() != function.params.len() {
                    panic!(
                        "Function '{}' expected {} arguments, got {}",
                        name,
                        function.params.len(),
                        arg_values.len()
                    );
                }

                let old_vars = self.variables.clone();
                self.variables.clear();

                for (param, val) in function.params.iter().zip(arg_values.into_iter()) {
                    self.variables.insert(param.clone(), val);
                }

                self.execute(function.body.clone());
                let result = self.get_result();
                self.variables = old_vars;

                result
            }
        }
    }

    fn handle_assign(&mut self, left: Expr, right: Expr) -> i32 {
        if let Expr::Variable(name) = left {
            let val = self.eval_expr(right);
            self.variables.insert(name, val);
            val
        } else {
            panic!("Left-hand side of assignment must be a variable");
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    /// Helper function to run a piece of C4 code and return the result.
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
    fn test_arithmetic() {
        let code = "return 2 + 3 * 4;";
        assert_eq!(run(code), 14);
    }

    #[test]
    fn test_variable_assignment() {
        let code = "let x = 10; let y = x + 5; return y;";
        assert_eq!(run(code), 15);
    }

    #[test]
    fn test_function_call() {
        let code = "
            fn square(n) {
                return n * n;
            }
            let result = square(6);
            return result;
        ";
        assert_eq!(run(code), 36);
    }

    #[test]
    fn test_scope_isolation() {
        let code = "
            fn test(x) { return x + 1; }
            let x = 5;
            let y = test(10);
            return x + y;
        ";
        assert_eq!(run(code), 16);
    }

    #[test]
    fn test_recursion() {
        let code = "
            fn factorial(n) {
                if (n == 0) {
                    return 1;
                } else {
                    return n * factorial(n - 1);
                }
            }
            let result = factorial(5);
            return result;
        ";
        assert_eq!(run(code), 120);
    }

    #[test]
    fn test_if_else() {
        let code = "
            let x = 10;
            if (x < 5) {
                return 0;
            } else {
                return 1;
            }
        ";
        assert_eq!(run(code), 1);
    }

    #[test]
    fn test_while_loop() {
        let code = "
            let i = 0;
            let sum = 0;
            while (i < 5) {
                let sum = sum + i;
                let i = i + 1;
            }
            return sum;
        ";
        assert_eq!(run(code), 10);
    }

    #[test]
    fn test_nested_if_else() {
        let code = "
            let x = 10;
            if (x > 5) {
                if (x < 15) {
                    return 1;
                } else {
                    return 2;
                }
            } else {
                return 0;
            }
        ";
        assert_eq!(run(code), 1);
    }

    #[test]
    fn test_nested_while_loops() {
        let code = "
            let i = 0;
            let sum = 0;
            while (i < 3) {
                let j = 0;
                while (j < 2) {
                    let sum = sum + i + j;
                    let j = j + 1;
                }
                let i = i + 1;
            }
            return sum;
        ";
        assert_eq!(run(code), 9);
    }

    #[test]
    fn test_function_multiple_params() {
        let code = "
            fn add(a, b, c) {
                return a + b + c;
            }
            let result = add(1, 2, 3);
            return result;
        ";
        assert_eq!(run(code), 6);
    }

    #[test]
    fn test_variable_shadowing() {
        let code = "
            let x = 5;
            {
                let x = 10;
                return x;
            }
            return x;
        ";
        assert_eq!(run(code), 10);
    }

    #[test]
    fn test_boolean_logic() {
        let code = "
            let a = true;
            let b = false;
            if (a && !b) {
                return 1;
            } else {
                return 0;
            }
        ";
        assert_eq!(run(code), 1);
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_division_by_zero() {
        let code = "return 10 / 0;";
        run(code);
    }

    #[test]
    #[should_panic(expected = "Variable 'y' not found")]
    fn test_undefined_variable() {
        let code = "return y;";
        run(code);
    }

    #[test]
    fn test_recursive_function_multiple_params() {
        let code = "
            fn power(base, exp) {
                if (exp == 0) {
                    return 1;
                } else {
                    return base * power(base, exp - 1);
                }
            }
            let result = power(2, 3);
            return result;
        ";
        assert_eq!(run(code), 8);
    }

    #[test]
    fn test_empty_block() {
        let code = "
            {
            }
            return 42;
        ";
        assert_eq!(run(code), 42);
    }

    #[test]
    fn test_function_overwriting() {
        let code = "
            fn test() {
                return 1;
            }
            fn test() {
                return 2;
            }
            return test();
        ";
        assert_eq!(run(code), 2);
    }

    #[test]
fn test_implicit_let() {
    let code = "
        x = 7
        y = x + 3
        return y
    ";
    assert_eq!(run(code), 10);
}

    
}
