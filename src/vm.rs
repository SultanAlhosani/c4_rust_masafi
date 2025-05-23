use crate::ast::{BinOp, Expr, Stmt, UnOp, Type};
use std::collections::HashMap;

/// Represents a function in the language, including its name, parameters, body, and return type.
#[derive(Clone)]
#[allow(dead_code)]
pub struct Function {
    /// The name of the function.
    pub name: String,
    /// A list of parameter names for the function.
    pub params: Vec<String>,
    /// The body of the function, represented as a statement.
    pub body: Stmt,
    /// The return type of the function, which may be `None` for void functions.
    pub return_type: Option<Type>, // Optional: None for void
}

/// Represents the different values that can be used at runtime, such as integers, strings, and arrays.
#[derive(Debug, Clone)]
pub enum Value {
    /// Integer value (e.g., 42)
    Int(i32),
    /// String value (e.g., "Hello")
    Str(String),
    /// Array value, which contains a vector of `Value`s.
    Array(Vec<Value>),
}

/// The virtual machine (VM) that runs the program, holding state like variables, functions, and constants.
pub struct Vm {
    /// The last result returned by an expression evaluation.
    pub last_result: Value,
    /// The list of variable scopes, with each scope being a map of variable names to values.
    pub variables: Vec<HashMap<String, Value>>,
    /// A map of function names to their corresponding function definitions.
    pub functions: HashMap<String, Function>,
    /// A map of constant names to their corresponding constant values.
    pub constants: HashMap<String, i32>,
    /// A flag that indicates whether the VM should return after the next statement.
    pub should_return: bool,
}

impl Vm {
    /// Creates a new instance of the virtual machine with initialized state.
    ///
    /// # Returns
    /// A new `Vm` instance with empty variables, functions, and constants.
    pub fn new() -> Self {
        Self {
            last_result: Value::Int(0),
            variables: vec![HashMap::new()],
            functions: HashMap::new(),
            constants: HashMap::new(),
            should_return: false,
        }
    }

    /// Sets the last result to a given value and sets the return flag to true.
    ///
    /// # Parameters
    /// - `value`: The value to set as the result.
    pub fn set_result(&mut self, value: Value) {
        self.last_result = value;
        self.should_return = true;
    }

    /// Retrieves the last result as an integer.
    ///
    /// # Returns
    /// The last result as an integer, or 0 if the result is not an integer.
    pub fn get_result(&self) -> i32 {
        match &self.last_result {
            Value::Int(i) => *i,
            Value::Str(_) => 0,
            Value::Array(_) => 0, // Default to 0 for arrays
        }
    }

    /// Retrieves the last result as a string.
    ///
    /// # Returns
    /// An `Option` containing the string, or `None` if the result is not a string.
    pub fn get_result_str(&self) -> Option<&str> {
        match &self.last_result {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Executes a statement, performing the appropriate actions based on the statement type.
    ///
    /// # Parameters
    /// - `stmt`: The statement to execute.
    pub fn execute(&mut self, stmt: Stmt) {
        if self.should_return {
            return;
        }

        match stmt {
            Stmt::Return(expr) => {
                let value = self.eval_expr(expr);
                self.set_result(value);
            }
            Stmt::Let { name, value, .. } => {
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
                    self.variables.pop();
                }
            }
            #[allow(unused_variables)]
            Stmt::Function { name, params, body, return_type } => {
                self.functions.insert(name.clone(), Function {
                    name,
                    params,
                    body: *body,
                    return_type: None, // or Some(Type::Int) if you want to default to int
                });
            }
            Stmt::Print(expr) => {
                let val = self.eval_expr(expr);
                match val {
                    Value::Int(i) => println!("{}", i),
                    Value::Str(s) => println!("{}", s),
                    Value::Array(arr) => {
                        let display = arr.iter()
                                         .map(|v| match v {
                                             Value::Int(i) => i.to_string(),
                                             Value::Str(s) => format!("\"{}\"", s),
                                             _ => String::from("?"),
                                         })
                                         .collect::<Vec<_>>()
                                         .join(", ");
                        println!("[{}]", display);
                    }
                }
            }
            Stmt::ExprStmt(expr) => {
                self.eval_expr(expr);
            }
        }
    }

    /// Evaluates an expression and returns its result as a `Value`.
    ///
    /// # Parameters
    /// - `expr`: The expression to evaluate.
    ///
    /// # Returns
    /// The evaluated result as a `Value`.
    fn eval_expr(&mut self, expr: Expr) -> Value {
        match expr {
            Expr::Number(n) => Value::Int(n),
            Expr::Boolean(b) => Value::Int(if b { 1 } else { 0 }),
            Expr::Char(c) => Value::Int(c as i32),
            Expr::StringLiteral(s) => Value::Str(s),
            Expr::Ternary { condition, then_branch, else_branch } => {
                if self.eval_as_bool(*condition) {
                    self.eval_expr(*then_branch)
                } else {
                    self.eval_expr(*else_branch)
                }
            }
            Expr::AddressOf(expr) => {
                let val = self.eval_expr(*expr);
                match val {
                    Value::Int(i) => Value::Int(i * 1000),
                    _ => panic!("Cannot take address of non-int"),
                }
            }
            Expr::Deref(expr) => {
                let addr = self.eval_expr(*expr);
                match addr {
                    Value::Int(fake_ptr) => Value::Int(fake_ptr / 1000),
                    _ => panic!("Invalid pointer dereference"),
                }
            }
            Expr::ArrayLiteral(elements) => {
                let evaluated = elements.into_iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Vec<_>>();
                Value::Array(evaluated)
            }
            Expr::ArrayIndex(array_expr, index_expr) => {
                let array_val = self.eval_expr(*array_expr);
                let index_val = self.eval_expr(*index_expr);
                let idx = match index_val {
                    Value::Int(i) => i as usize,
                    _ => panic!("Array index must be an integer"),
                };
                match array_val {
                    Value::Array(vec) => vec.get(idx).cloned().unwrap_or_else(|| {
                        panic!("Array index out of bounds: {}", idx)
                    }),
                    _ => panic!("Attempted to index non-array value"),
                }
            }
            Expr::PreInc(expr) => {
                if let Expr::Variable(name) = *expr {
                    for scope in self.variables.iter_mut().rev() {
                        if let Some(Value::Int(ref mut val)) = scope.get_mut(&name) {
                            *val += 1;
                            return Value::Int(*val);
                        }
                    }
                    panic!("Variable '{}' not found", name);
                } else {
                    panic!("++ requires a variable");
                }
            }
            Expr::PreDec(expr) => {
                if let Expr::Variable(name) = *expr {
                    for scope in self.variables.iter_mut().rev() {
                        if let Some(Value::Int(ref mut val)) = scope.get_mut(&name) {
                            *val -= 1;
                            return Value::Int(*val);
                        }
                    }
                    panic!("Variable '{}' not found", name);
                } else {
                    panic!("-- requires a variable");
                }
            }
            Expr::PostInc(expr) => {
                if let Expr::Variable(name) = *expr {
                    for scope in self.variables.iter_mut().rev() {
                        if let Some(Value::Int(ref mut val)) = scope.get_mut(&name) {
                            let original = *val;
                            *val += 1;
                            return Value::Int(original);
                        }
                    }
                    panic!("Variable '{}' not found", name);
                } else {
                    panic!("++ requires a variable");
                }
            }
            Expr::PostDec(expr) => {
                if let Expr::Variable(name) = *expr {
                    for scope in self.variables.iter_mut().rev() {
                        if let Some(Value::Int(ref mut val)) = scope.get_mut(&name) {
                            let original = *val;
                            *val -= 1;
                            return Value::Int(original);
                        }
                    }
                    panic!("Variable '{}' not found", name);
                } else {
                    panic!("-- requires a variable");
                }
            }
            Expr::SizeOf(t) => {
                let size: i32 = match t {
                    Type::Int => 4,
                    Type::Char => 1,
                    Type::Pointer(_) => 8,
                    Type::Void => 0,
                    Type::Array(elem_type, len) => {
                        let elem_size = match *elem_type {
                            Type::Int => 4,
                            Type::Char => 1,
                            Type::Pointer(_) => 8,
                            Type::Void => 0,
                            Type::Array(_, _) => panic!("Nested arrays not supported in sizeof"),
                        };
                        elem_size * (len as i32)
                    }
                };
                
                Value::Int(size)
            }
            Expr::Cast(to_type, expr) => {
                let val = self.eval_expr(*expr);
                match (&to_type, val) {
                    (Type::Int, Value::Int(i)) => Value::Int(i),
                    (Type::Char, Value::Int(i)) => Value::Int(i & 0xFF),
                    (Type::Int, Value::Str(_)) => Value::Int(0),
                    (Type::Char, Value::Str(_)) => Value::Int(0),
                    (Type::Pointer(_), Value::Int(i)) => Value::Int(i),
                    (_, v) => panic!("Unsupported cast: {:?} to {:?}", v, to_type),
                }
            }
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
                        BinOp::Mod => {
                            if ri == 0 {
                                panic!("Modulo by zero");
                            }
                            Value::Int(li % ri)
                        }
                        BinOp::Equal => Value::Int((li == ri) as i32),
                        BinOp::NotEqual => Value::Int((li != ri) as i32),
                        BinOp::LessThan => Value::Int((li < ri) as i32),
                        BinOp::GreaterThan => Value::Int((li > ri) as i32),
                        BinOp::LessEqual => Value::Int((li <= ri) as i32),
                        BinOp::GreaterEqual => Value::Int((li >= ri) as i32),
                        BinOp::And => Value::Int((li != 0 && ri != 0) as i32),
                        BinOp::Or => Value::Int((li != 0 || ri != 0) as i32),
                        BinOp::BitAnd => Value::Int(li & ri),
                        BinOp::BitOr => Value::Int(li | ri),
                        BinOp::BitXor => Value::Int(li ^ ri),
                        BinOp::Shl => Value::Int(li << ri),
                        BinOp::Shr => Value::Int(li >> ri),
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
                        Value::Array(_) => panic!("Cannot apply 'Not' operator to an array"),
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

    /// Handles assignment operations for variables and array indices.
    ///
    /// # Parameters
    /// - `left`: The left-hand side expression (either a variable or an array index).
    /// - `right`: The value to assign.
    ///
    /// # Returns
    /// The value that was assigned to the left-hand side.
    fn handle_assign(&mut self, left: Expr, right: Expr) -> Value {
        match left {
            Expr::Variable(name) => {
                let val = self.eval_expr(right);
                for scope in self.variables.iter_mut().rev() {
                    if scope.contains_key(&name) {
                        scope.insert(name.clone(), val.clone());
                        return val;
                    }
                }
                self.variables.last_mut().unwrap().insert(name, val.clone());
                val
            }
            Expr::ArrayIndex(array_expr, index_expr) => {
                let array_name = match *array_expr {
                    Expr::Variable(name) => name,
                    _ => panic!("Left-hand side must be a variable array reference"),
                };
                let index = match self.eval_expr(*index_expr) {
                    Value::Int(i) => i as usize,
                    _ => panic!("Array index must be an integer"),
                };
                let val = self.eval_expr(right);
                for scope in self.variables.iter_mut().rev() {
                    if let Some(Value::Array(ref mut vec)) = scope.get_mut(&array_name) {
                        if index >= vec.len() {
                            panic!("Array index {} out of bounds", index);
                        }
                        vec[index] = val.clone();
                        return val;
                    }
                }
                panic!("Array '{}' not found", array_name);
            }
            _ => panic!("Left-hand side of assignment must be a variable or array element"),
        }
    }

    /// Evaluates an expression and returns its result as a boolean value.
    ///
    /// # Parameters
    /// - `expr`: The expression to evaluate.
    ///
    /// # Returns
    /// A boolean value (`true` or `false`).
    fn eval_as_bool(&mut self, expr: Expr) -> bool {
        match self.eval_expr(expr) {
            Value::Int(i) => i != 0,  // Non-zero integers are treated as true, zero as false
            Value::Str(_) => true,     // Any non-empty string is considered "truthy"
            Value::Array(_) => true,   // Arrays are considered "truthy"
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    /// Helper function to run a piece of C4 code and return the result.
    ///
    /// # Parameters
    /// - `code`: A string containing the C4 code to execute.
    ///
    /// # Returns
    /// The integer result of executing the C4 code.
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

    /// Tests basic arithmetic operations such as addition and multiplication.
    #[test]
    fn test_arithmetic() {
        let code = "return 2 + 3 * 4;";
        assert_eq!(run(code), 14);
    }

    /// Tests variable assignment and using those variables in calculations.
    #[test]
    fn test_variable_assignment() {
        let code = "let x = 10; let y = x + 5; return y;";
        assert_eq!(run(code), 15);
    }

    /// Tests function call with a single parameter and return value.
    #[test]
    fn test_function_call() {
        let code = "
            int square(n) {
                return n * n;
            }
            let result = square(6);
            return result;
        ";
        assert_eq!(run(code), 36);
    }

    /// Tests scope isolation in functions and variables.
    #[test]
    fn test_scope_isolation() {
        let code = "
            int test(x) { return x + 1; }
            let x = 5;
            let y = test(10);
            return x + y;
        ";
        assert_eq!(run(code), 16);
    }

    /// Tests recursion in functions, specifically calculating a factorial.
    #[test]
    fn test_recursion() {
        let code = "
            int factorial(n) {
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

    /// Tests if-else conditionals with basic true/false evaluations.
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

    /// Tests while loop for sum calculation with a break condition.
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

    /// Tests nested if-else statements.
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

    /// Tests nested while loops for a sum calculation.
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

    /// Tests function call with multiple parameters.
    #[test]
    fn test_function_multiple_params() {
        let code = "
            int add(a, b, c) {
                return a + b + c;
            }
            let result = add(1, 2, 3);
            return result;
        ";
        assert_eq!(run(code), 6);
    }

    /// Tests variable shadowing by declaring variables with the same name in different scopes.
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

    /// Tests boolean logic with `&&` and `!` operators.
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

    /// Tests division by zero, expecting a panic.
    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_division_by_zero() {
        let code = "return 10 / 0;";
        run(code);
    }

    /// Tests accessing an undefined variable, expecting a panic.
    #[test]
    #[should_panic(expected = "Variable 'y' not found")]
    fn test_undefined_variable() {
        let code = "return y;";
        run(code);
    }

    /// Tests recursion with multiple parameters in a function, such as power calculation.
    #[test]
    fn test_recursive_function_multiple_params() {
        let code = "
            int power(base, exp) {
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

    /// Tests an empty block of code, ensuring it doesn't cause issues.
    #[test]
    fn test_empty_block() {
        let code = "
            {
            }
            return 42;
        ";
        assert_eq!(run(code), 42);
    }

    /// Tests function overwriting, where a function with the same name is defined twice.
    #[test]
    fn test_function_overwriting() {
        let code = "
            int test() {
                return 1;
            }
            int test() {
                return 2;
            }
            return test();
        ";
        assert_eq!(run(code), 2);
    }

    /// Tests implicit variable declarations without the `let` keyword.
    #[test]
    fn test_implicit_let() {
        let code = "
            x = 7;
            y = x + 3;
            return y;
        ";
        assert_eq!(run(code), 10);
    }

    /// Tests global variable usage inside a function.
    #[test]
    fn test_global_variable_usage() {
        let code = "
            let x = 42;

            int show() {
                return x;
            }

            return show();
        ";
        assert_eq!(run(code), 42);
    }

    /// Tests string concatenation and printing.
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

    /// Tests global variable access inside a function.
    #[test]
    fn test_global_variable_access_in_function() {
        let code = r#"
            let x = 123;

            int get() {
                return x;
            }

            return get();
        "#;

        assert_eq!(run(code), 123);
    }

    /// Tests modifying a global variable inside a function.
    #[test]
    fn test_global_variable_modification_in_function() {
        let code = r#"
            let x = 10;

            int modify() {
                x = x + 5;
            }

            modify();
            return x;
        "#;

        assert_eq!(run(code), 15);
    }

    /// Tests global variable shadowing within nested blocks.
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

    /// Tests comma-separated variable declarations in a single statement.
    #[test]
    fn test_comma_separated_let_declaration() {
        let code = "
            let x = 1, y = 2, z = x + y;
            return z;
        ";
        assert_eq!(run(code), 3);
    }

    /// Tests parentheses overriding the default precedence in expressions.
    #[test]
    fn test_parentheses_override_precedence() {
        let code = "
            let a = 2;
            let b = 3;
            let c = 4;
            let d = 20;
            return (a + b) * c == d;
        ";
        assert_eq!(run(code), 1);
    }

    /// Tests the `sizeof` operator for different types.
    #[test]
    fn test_sizeof_expression() {
        let code = "
            return sizeof(int);
        ";
        assert_eq!(run(code), 4);
    }

    /// Tests `sizeof` for multiple types.
    #[test]
    fn test_sizeof_multiple_types() {
        assert_eq!(run("return sizeof(char);"), 1);
        assert_eq!(run("return sizeof(bool);"), 1);
        assert_eq!(run("return sizeof(str);"), 8);
    }

    /// Tests parsing and using enums in the language.
    #[test]
    fn test_enum_parsing_and_usage() {
        let code = "
            enum { A = 5, B, C = 10, D };
            return A + B + C + D; // 5 + 6 + 10 + 11 = 32
        ";

        let lexer = Lexer::new(code);
        let mut vm = Vm::new();
        let mut parser = Parser::new(lexer, &mut vm);
        let stmts = parser.parse();
        for stmt in stmts {
            vm.execute(stmt);
        }

        assert_eq!(vm.get_result(), 32);
    }

    /// Tests type casting with different types.
    #[test]
    fn test_type_casting() {
        let code = r#"
            let x = (int)"hello";
            let y = (char)300;
            let z = (int)123;
            return x + y + z;
        "#;
        assert_eq!(run(code), 167);
    }

    /// Tests printing from the main function.
    #[test]
    fn test_print_from_main() {
        let code = r#"
            void greet() {
                print("Hello from C4!");
            }

            int main() {
                greet();
                return 42;
            }

            return main();
        "#;

        let lexer = Lexer::new(code);
        let mut vm = Vm::new();
        let mut parser = Parser::new(lexer, &mut vm);
        let stmts = parser.parse();
        for stmt in stmts {
            vm.execute(stmt);
        }

        assert_eq!(vm.get_result(), 42);
    }

    /// Tests pre- and post-increment operators.
    #[test]
    fn test_pre_post_increment() {
        let code = "
            let x = 5;
            let a = ++x;  // x = 6, a = 6
            let b = x++;  // b = 6, x = 7
            return a + b + x;  // returns 19
        ";
        assert_eq!(run(code), 19);
    }

    /// Tests pre- and post-decrement operators.
    #[test]
    fn test_pre_post_decrement() {
        let code = "
            let x = 10;
            let a = --x;  // x = 9, a = 9
            let b = x--;  // b = 9, x = 8
            return a + b + x;  // 9 + 9 + 8 = 26
        ";
        assert_eq!(run(code), 26);
    }

    /// Tests enum parsing with automatic increments.
    #[test]
    fn test_enum_parsing_auto_increment() {
        let code = "
            enum { A = 10, B, C = 20, D };
            return A + B + C + D;
        ";

        let lexer = Lexer::new(code);
        let mut vm = Vm::new();
        let mut parser = Parser::new(lexer, &mut vm);
        let stmts = parser.parse();

        for stmt in stmts {
            vm.execute(stmt);
        }

        assert_eq!(vm.get_result(), 62);
    }

    /// Tests modulus operation in expressions.
    #[test]
    fn test_modulus() {
        let code = "return 10 % 3;";
        assert_eq!(run(code), 1);
    }

    /// Tests bitwise operations like AND, OR, XOR, and shifts.
    #[test]
    fn test_bitwise_operations() {
        let code = "
            let a = 6;      // 0b0110
            let b = 3;      // 0b0011
            let and = a & b;    // 0b0010 -> 2
            let or  = a | b;    // 0b0111 -> 7
            let xor = a ^ b;    // 0b0101 -> 5
            let shl = a << 1;   // 0b1100 -> 12
            let shr = a >> 1;   // 0b0011 -> 3
            return and + or + xor + shl + shr; // 2 + 7 + 5 + 12 + 3 = 29
        ";
        assert_eq!(run(code), 29);
    }

    /// Tests printing an array.
    #[test]
    fn test_print_array() {
        let code = r#"
            let x = [1, 2, 3];
            print(x); // should print: [1, 2, 3]
            return x[1]; // return middle element to confirm indexing
        "#;

        let lexer = Lexer::new(code);
        let mut vm = Vm::new();
        let mut parser = Parser::new(lexer, &mut vm);
        let stmts = parser.parse();

        for stmt in stmts {
            vm.execute(stmt);
        }

        assert_eq!(vm.get_result(), 2); // confirm array indexing works
    }

    /// Tests fake pointer dereferencing.
    #[test]
    fn test_pointer_fake_deref() {
        let code = "
            let x = 42;
            let p = &x;
            let y = *p;
            return y;
        ";
        assert_eq!(run(code), 42);
    }

    /// Tests identity of pointers.
    #[test]
    fn test_pointer_identity() {
        let code = "
            let x = 123;
            let ptr = &x;
            return *ptr + 1;
        ";
        assert_eq!(run(code), 124);
    }

    /// Tests array assignment functionality.
    #[test]
    fn test_array_assignment() {
        let code = "
            let arr = [0, 0, 0];
            arr[1] = 42;
            return arr[1];
        ";
        assert_eq!(run(code), 42);
    }

    /// Tests handling of nested arrays in `sizeof`.
    #[test]
    #[should_panic(expected = "Nested arrays not supported")]
    fn test_nested_array_sizeof() {
        let code = "return sizeof(int[3][2]);";
        run(code);
    }

    /// Tests pointer casting in expressions.
    #[test]
    fn test_pointer_casting() {
        let code = "
            let x = 5;
            let ptr = &x;
            let val = (int)ptr;
            return val / 1000;  // should return original value of x
        ";
        assert_eq!(run(code), 5);
    }

    /// Tests using enums inside functions.
    #[test]
    fn test_enum_inside_function() {
        let code = "
            enum { A = 1, B = 2 };
            int sum() {
                return A + B;
            }
            return sum();
        ";
        assert_eq!(run(code), 3);
    }
}
