use crate::ast::{BinOp, Expr, Stmt, UnOp};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Stmt,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Str(String),
}

pub struct Vm {
    pub last_result: Value,
    pub variables: Vec<HashMap<String, Value>>,
    pub functions: HashMap<String, Function>,
    pub constants: HashMap<String, i32>,
    pub should_return: bool,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            last_result: Value::Int(0),
            variables: vec![HashMap::new()],
            functions: HashMap::new(),
            constants: HashMap::new(),
            should_return: false,
        }
    }

    pub fn set_result(&mut self, value: Value) {
        self.last_result = value;
        self.should_return = true;
    }

    pub fn get_result(&self) -> i32 {
        match &self.last_result {
            Value::Int(i) => *i,
            Value::Str(_) => 0,
        }
    }

    pub fn get_result_str(&self) -> Option<&str> {
        match &self.last_result {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    

    pub fn execute(&mut self, stmt: Stmt) {
        if self.should_return {
            return;
        }

        match stmt {
            Stmt::Return(expr) => {
                let value = self.eval_expr(expr);
                self.set_result(value);
            }
            Stmt::Let { name, value } => {
                let val = self.eval_expr(value);
                self.variables.last_mut().unwrap().insert(name, val);
            }
            Stmt::Assign { name, value } => {
                let val = self.eval_expr(value);
                for scope in self.variables.iter_mut().rev() {
                    if scope.contains_key(&name) {
                        scope.insert(name, val);
                        return;
                    }
                }
                self.variables.last_mut().unwrap().insert(name, val);
            }
            Stmt::If { condition, then_branch, else_branch } => {
                if self.eval_as_bool(condition) {
                    self.execute(*then_branch);
                } else if let Some(else_stmt) = else_branch {
                    self.execute(*else_stmt);
                }
            }
            Stmt::While { condition, body } => {
                while self.eval_as_bool(condition.clone()) {
                    self.execute(*body.clone());
                    if self.should_return {
                        break;
                    }
                }
            }
            Stmt::Block(stmts) => {
                let is_single_scope = stmts.iter().all(|s| matches!(s, Stmt::Let { .. }));
                
                if !is_single_scope {
                    self.variables.push(HashMap::new());
                }
            
                for stmt in stmts {
                    self.execute(stmt);
                    if self.should_return {
                        break;
                    }
                }
            
                if !is_single_scope {
                    self.variables.pop(); // only pop if you pushed
                }
            }
            
            Stmt::Function { name, params, body } => {
                self.functions.insert(name.clone(), Function { name, params, body: *body });
            }
            Stmt::Print(expr) => {
                let val = self.eval_expr(expr);
                match val {
                    Value::Int(i) => println!("{}", i),
                    Value::Str(s) => println!("{}", s),
                }
            }
            Stmt::ExprStmt(expr) => {
                self.eval_expr(expr);
            }
        }
    }

    fn eval_as_bool(&mut self, expr: Expr) -> bool {
        match self.eval_expr(expr) {
            Value::Int(i) => i != 0,
            Value::Str(_) => true,
        }
    }

    fn eval_expr(&mut self, expr: Expr) -> Value {
        match expr {
            Expr::Number(n) => Value::Int(n),
            Expr::Boolean(b) => Value::Int(if b { 1 } else { 0 }),
            Expr::Char(c) => Value::Int(c as i32),
            Expr::StringLiteral(s) => Value::Str(s),
            Expr::Variable(name) => {
                for scope in self.variables.iter().rev() {
                    if let Some(val) = scope.get(&name) {
                        return val.clone();
                    }
                }
                if let Some(i) = self.constants.get(&name) {
                    return Value::Int(*i);
                }
                panic!("Variable '{}' not found", name);
            }
            Expr::EnumValue(enum_name, variant_name) => {
                let key = format!("{}::{}", enum_name, variant_name);
                Value::Int(*self.constants.get(&key).unwrap_or_else(|| {
                    panic!("Enum variant '{}' not found", key)
                }))
            }
            Expr::BinaryOp { op, left, right } => {
                if op == BinOp::Assign {
                    return self.handle_assign(*left, *right);
                }
                let l = self.eval_expr(*left);
                let r = self.eval_expr(*right);
                match (l, r) {
                    (Value::Int(li), Value::Int(ri)) => match op {
                        BinOp::Add => Value::Int(li + ri),
                        BinOp::Sub => Value::Int(li - ri),
                        BinOp::Mul => Value::Int(li * ri),
                        BinOp::Div => {
                            if ri == 0 {
                                panic!("Division by zero");
                            }
                            Value::Int(li / ri)
                        }
                        BinOp::Equal => Value::Int((li == ri) as i32),
                        BinOp::NotEqual => Value::Int((li != ri) as i32),
                        BinOp::LessThan => Value::Int((li < ri) as i32),
                        BinOp::GreaterThan => Value::Int((li > ri) as i32),
                        BinOp::LessEqual => Value::Int((li <= ri) as i32),
                        BinOp::GreaterEqual => Value::Int((li >= ri) as i32),
                        BinOp::And => Value::Int((li != 0 && ri != 0) as i32),
                        BinOp::Or => Value::Int((li != 0 || ri != 0) as i32),
                        _ => unreachable!(),
                    },
                    (Value::Str(ls), Value::Str(rs)) => match op {
                        BinOp::Add => Value::Str(ls + &rs),
                        BinOp::Equal => Value::Int((ls == rs) as i32),
                        BinOp::NotEqual => Value::Int((ls != rs) as i32),
                        _ => panic!("Unsupported string operation: {:?}", op),
                    },
                    _ => panic!("Mismatched types for operation"),
                }
            }
            Expr::UnaryOp { op, expr } => {
                let val = self.eval_expr(*expr);
                match op {
                    UnOp::Not => match val {
                        Value::Int(i) => Value::Int(if i == 0 { 1 } else { 0 }),
                        Value::Str(_) => Value::Int(0),
                    },
                }
            }
            Expr::FunctionCall { name, args } => {
                let function = self.functions.get(&name).unwrap_or_else(|| {
                    panic!("Function '{}' not found", name)
                }).clone();

                let arg_values: Vec<Value> = args.into_iter().map(|arg| self.eval_expr(arg)).collect();

                if arg_values.len() != function.params.len() {
                    panic!(
                        "Function '{}' expected {} arguments, got {}",
                        name,
                        function.params.len(),
                        arg_values.len()
                    );
                }

                self.variables.push(HashMap::new());
                for (param, val) in function.params.iter().zip(arg_values) {
                    self.variables.last_mut().unwrap().insert(param.clone(), val);
                }

                let prev_result = self.last_result.clone();
                let prev_should_return = self.should_return;
                self.last_result = Value::Int(0);
                self.should_return = false;

                self.execute(function.body.clone());

                let result = self.last_result.clone();
                self.variables.pop();
                self.last_result = prev_result;
                self.should_return = prev_should_return;
                result
            }
        }
    }

    fn handle_assign(&mut self, left: Expr, right: Expr) -> Value {
        if let Expr::Variable(name) = left {
            let val = self.eval_expr(right);
            for scope in self.variables.iter_mut().rev() {
                if scope.contains_key(&name) {
                    scope.insert(name, val.clone());
                    return val;
                }
            }
            self.variables.last_mut().unwrap().insert(name, val.clone());
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
                    return n * factorial(n - 1); // <-- semicolon OK
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
                sum = sum + i;
                i = i + 1;

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
                    sum = sum + i + j;
                    j = j + 1;
                }
                i = i + 1;
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
            x = 7;
            y = x + 3;
            return y;

    ";
    assert_eq!(run(code), 10);
}

#[test]
fn test_global_variable_usage() {
    let code = "
        let x = 42;

        fn show() {
            return x;
        }

        return show();
    ";
    assert_eq!(run(code), 42);
}

#[test]
fn test_string_return_and_concatenation() {
    let code = r#"
        let hello = "Hello, ";
        let world = "World!";
        let message = hello + world;
        print(message);
        return message;
    "#;

    let lexer = Lexer::new(code);
    let mut vm = Vm::new();
    let mut parser = Parser::new(lexer, &mut vm);
    let stmts = parser.parse();
    for stmt in stmts {
        vm.execute(stmt);
    }

    match vm.last_result {
        Value::Str(ref s) => assert_eq!(s, "Hello, World!"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_global_variable_access_in_function() {
    let code = r#"
        let x = 123;

        fn get() {
            return x;
        }

        return get();
    "#;

    assert_eq!(run(code), 123);
}


#[test]
fn test_global_variable_modification_in_function() {
    let code = r#"
        let x = 10;

        fn modify() {
            x = x + 5;
        }

        modify();
        return x;
    "#;

    assert_eq!(run(code), 15);
}

#[test]
fn test_global_variable_shadowing() {
    let code = r#"
        let x = 7;

        {
            let x = 42;
            print(x); // should print 42
        }

        return x; // should return 7
    "#;

    assert_eq!(run(code), 7);
}

#[test]
fn test_comma_separated_let_declaration() {
    let code = "
        let x = 1, y = 2, z = x + y;
        return z;
    ";
    assert_eq!(run(code), 3);
}

    
}
