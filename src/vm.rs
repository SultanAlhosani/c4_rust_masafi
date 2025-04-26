use crate::ast::{Stmt, Expr, BinOp};

pub struct Vm {
    pub last_result: i32,
}

impl Vm {
    pub fn new() -> Self {
        Self { last_result: 0 }
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
        }
    }
    
    
    

    fn eval_expr(&self, expr: Expr) -> i32 {
        match expr {
            Expr::Number(n) => n,
            Expr::BinaryOp { op, left, right } => {
                let l = self.eval_expr(*left);
                let r = self.eval_expr(*right);
                match op {
                    BinOp::Add => l + r,
                    BinOp::Sub => l - r,
                    BinOp::Mul => l * r,
                    BinOp::Div => l / r,
                }
            }
        }
    }
}
