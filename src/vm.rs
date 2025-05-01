use crate::ast::{BinOp, Expr, Stmt};
use std::collections::HashMap;

/// Represents a user-defined function in the virtual machine.
///
/// # Fields
/// - `name`: The name of the function.
/// - `params`: The list of parameter names.
/// - `body`: The body of the function as a statement.
#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Stmt,
}

/// The virtual machine (VM) responsible for executing statements and evaluating expressions.
///
/// The VM maintains a state that includes variables, functions, and the result of the last executed statement.
pub struct Vm {
    pub last_result: i32,
    pub variables: HashMap<String, i32>,
    pub functions: HashMap<String, Function>,
}

impl Vm {
    /// Creates a new `Vm` instance.
    ///
    /// # Returns
    /// A new `Vm` instance with an empty state.
    pub fn new() -> Self {
        Self {
            last_result: 0,
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Sets the result of the last executed statement.
    ///
    /// # Parameters
    /// - `value`: The value to set as the result.
    pub fn set_result(&mut self, value: i32) {
        self.last_result = value;
    }

    /// Retrieves the result of the last executed statement.
    ///
    /// # Returns
    /// The value of the last result.
    pub fn get_result(&self) -> i32 {
        self.last_result
    }

    /// Executes a statement in the virtual machine.
    ///
    /// # Parameters
    /// - `stmt`: The statement to execute.
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
                if self.variables.contains_key(&name) {
                    self.variables.insert(name, val);
                } else {
                    panic!("Variable '{}' not declared", name);
                }
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
        }
    }

    /// Evaluates an expression in the virtual machine.
    ///
    /// # Parameters
    /// - `expr`: The expression to evaluate.
    ///
    /// # Returns
    /// The result of the evaluated expression as an integer.
    fn eval_expr(&mut self, expr: Expr) -> i32 {
        match expr {
            Expr::Number(n) => n,
            Expr::Boolean(b) => {
                if b {
                    1
                } else {
                    0
                }
            }
            Expr::Char(c) => c as i32,
            Expr::Variable(name) => *self
                .variables
                .get(&name)
                .expect(&format!("Variable '{}' not found", name)),
            Expr::BinaryOp { op, left, right } => {
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
                    BinOp::Equal => {
                        if l == r {
                            1
                        } else {
                            0
                        }
                    }
                    BinOp::NotEqual => {
                        if l != r {
                            1
                        } else {
                            0
                        }
                    }
                    BinOp::LessThan => {
                        if l < r {
                            1
                        } else {
                            0
                        }
                    }
                    BinOp::GreaterThan => {
                        if l > r {
                            1
                        } else {
                            0
                        }
                    }
                }
            }
            Expr::FunctionCall { name, args } => {
                // Clone the function to avoid borrow conflicts
                let function = self
                    .functions
                    .get(&name)
                    .expect(&format!("Function '{}' not found", name))
                    .clone();

                // Evaluate arguments
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

                // Save the current variable scope
                let old_vars = self.variables.clone();
                self.variables.clear();

                // Set up function arguments in the new scope
                for (param, val) in function.params.iter().zip(arg_values.into_iter()) {
                    self.variables.insert(param.clone(), val);
                }

                // Execute the function body
                self.execute(function.body.clone());
                let result = self.get_result();

                // Restore the old variable scope
                self.variables = old_vars;

                result
            }
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
}
