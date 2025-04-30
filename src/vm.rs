use crate::ast::{BinOp, Expr, Stmt};
use std::collections::HashMap;

// ✅ Derive Clone for Function struct
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
                // ✅ Clone the function first to break the borrow early
                let function = self
                    .functions
                    .get(&name)
                    .expect(&format!("Function '{}' not found", name))
                    .clone();

                // ✅ Evaluate arguments now that borrow is done
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

                // ✅ Save old scope
                let old_vars = self.variables.clone();
                self.variables.clear();

                // ✅ Set up function args in scope
                for (param, val) in function.params.iter().zip(arg_values.into_iter()) {
                    self.variables.insert(param.clone(), val);
                }

                // ✅ Execute function
                self.execute(function.body.clone());
                let result = self.get_result();

                // ✅ Restore old scope
                self.variables = old_vars;

                result
            }
        }
    }
}
