use crate::ast::{Stmt, Expr, BinOp};
use std::collections::HashMap;

pub struct Vm {
    pub last_result: i32,
    pub variables: HashMap<String, i32>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            last_result: 0,
            variables: HashMap::new(),
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
            Stmt::If { condition, then_branch, else_branch } => {
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
        }
    }

    fn eval_expr(&self, expr: Expr) -> i32 {
        match expr {
            Expr::Number(n) => n,
            Expr::Boolean(b) => if b { 1 } else { 0 },
            Expr::Char(c) => c as i32, // <-- NEW: characters are evaluated as their ASCII codes
            Expr::Variable(name) => {
                *self.variables.get(&name).expect(&format!("Variable '{}' not found", name))
            }
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
                    BinOp::Equal => if l == r { 1 } else { 0 },
                    BinOp::NotEqual => if l != r { 1 } else { 0 },
                    BinOp::LessThan => if l < r { 1 } else { 0 },
                    BinOp::GreaterThan => if l > r { 1 } else { 0 },
                }
            }
        }
    }
}
