use crate::ast::{Expr, Stmt, BinOp, UnOp, Type}; // Import AST types (expressions, statements, etc.)
use crate::lexer::{Lexer, Token}; // Import Lexer and Token definitions
use crate::vm::Vm; // Import the VM module for code execution
use std::collections::HashMap; // Import HashMap for storing type mappings

/// The `Parser` struct is responsible for parsing the input source code
/// into an intermediate representation that can be processed by the VM.
/// 
/// # Fields
/// 
/// - `lexer`: An instance of the `Lexer` used to tokenize the input source code.
/// - `current_token`: The current token being processed by the parser.
/// - `vm`: A mutable reference to the `Vm` instance, which executes the parsed code.
/// - `type_map`: A `HashMap` that maps type names (as `String`) to their corresponding `Type` definitions.
pub struct Parser<'a> {
    lexer: Lexer, // Lexer instance to tokenize the input
    current_token: Token, // Current token to be processed
    vm: &'a mut Vm, // Reference to the virtual machine for execution
    type_map: HashMap<String, Type>, // A map for storing types (e.g., int, char, etc.)
}

impl<'a> Parser<'a> {
    /// Creates a new `Parser` instance.
    /// Initializes the parser, sets the current token to the first token, and prepares the type map.
    pub fn new(lexer: Lexer, vm: &'a mut Vm) -> Self {
        let mut parser = Self {
            lexer,
            current_token: Token::Eof, // Start with EOF (End of File) token
            vm,
            type_map: HashMap::new(), // Initialize the type map
        };
        parser.next(); // Move to the first token
        parser
    }

    /// Advances to the next token in the input.
    pub fn next(&mut self) {
        self.current_token = self.lexer.next_token(); // Get the next token from the lexer
    }

    /// Parses the entire input and returns a vector of statements.
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new(); // Initialize an empty vector to hold statements
        while self.current_token != Token::Eof { // Loop until EOF is encountered
            statements.push(self.statement()); // Parse each statement
        }
        statements
    }

    /// Parses a single statement from the input.
    /// It handles various kinds of statements (e.g., variable declarations, function declarations, etc.)
    fn statement(&mut self) -> Stmt {
        let (line, col) = self.lexer.get_position(); // Get the current position (line, column)

        // Check for function or typed variable declaration
        if let Token::Identifier(ref type_name) = self.current_token {
            if matches!(type_name.as_str(), "int" | "char" | "bool" | "str" | "void") {
                let var_type = self.parse_type().unwrap(); // Parse the variable type
                let (name_line, name_col) = self.lexer.get_position(); // Get position of the variable name
                let name = self.expect_identifier("Expected name after type", name_line, name_col); // Expect a valid identifier for variable name

                // If the next token is an opening parenthesis, it’s a function declaration
                if self.current_token == Token::OpenParen {
                    self.next();
                    let mut params = Vec::new(); // Initialize an empty vector for function parameters
                    while self.current_token != Token::CloseParen { // Parse parameters inside the parentheses
                        let param_name = self.expect_identifier("Expected parameter name", line, col);
                        params.push(param_name); // Add parameter name to the list
                        if self.current_token == Token::Comma {
                            self.next(); // Move past the comma
                        } else if self.current_token != Token::CloseParen {
                            panic!("Expected ',' or ')' in parameter list at line {}, column {}", line, col);
                        }
                    }
                    self.expect_token(Token::CloseParen, "Expected ')' after parameters", line, col); // Expect closing parenthesis
                    let body = Box::new(self.block()); // Parse the body of the function
                    return Stmt::Function {
                        name,
                        params,
                        body,
                        return_type: Some(var_type),
                    };
                } else {
                    // Handle variable declaration
                    self.expect_token(Token::Assign, "Expected '=' after variable name", line, col); // Expect assignment operator
                    let value = self.expression(); // Parse the expression on the right-hand side
                    self.type_map.insert(name.clone(), var_type.clone()); // Add variable type to the type map
                    self.expect_token(Token::Semicolon, "Expected ';' after variable declaration", line, col); // Expect semicolon
                    return Stmt::Let { name, value, var_type: Some(var_type) }; // Return a Let statement
                }
            }
        }

        match &self.current_token {
            // Handle different types of statements
            Token::Return => {
                self.next();
                let expr = if matches!(self.current_token, Token::Semicolon | Token::CloseBrace) {
                    Expr::Number(0) // If the next token is a semicolon or closing brace, return 0
                } else {
                    self.expression() // Otherwise, parse an expression
                };
                if self.current_token == Token::Semicolon {
                    self.next(); // Consume the semicolon
                }
                Stmt::Return(expr) // Return the parsed return statement
            }

            Token::Let => {
                self.next();
                let mut decls = Vec::new(); // Initialize an empty vector for declarations
                loop {
                    let name = self.expect_identifier("Expected identifier after 'let'", line, col); // Parse variable name
                    let var_type = if self.current_token == Token::Colon {
                        self.next();
                        self.parse_type().unwrap_or(Type::Int) // Parse type after colon
                    } else {
                        Type::Int // Default to int if no type specified
                    };
                    self.expect_token(Token::Assign, "Expected '=' after identifier", line, col); // Expect assignment
                    let value = self.expression(); // Parse the expression
                    self.type_map.insert(name.clone(), var_type.clone()); // Add variable to type map
                    decls.push(Stmt::Let { name, value, var_type: Some(var_type) }); // Add declaration to the list
                    if self.current_token == Token::Comma {
                        self.next(); // Consume the comma if present
                    } else {
                        break;
                    }
                }
                self.expect_token(Token::Semicolon, "Expected ';' after let", line, col); // Expect semicolon at the end
                if decls.len() == 1 {
                    decls.pop().unwrap() // Return single declaration
                } else {
                    Stmt::Block(decls) // Return block if multiple declarations
                }
            }

            Token::Print => {
                self.next();
                self.expect_token(Token::OpenParen, "Expected '(' after 'print'", line, col); // Expect opening parenthesis
                let expr = self.expression(); // Parse the expression to print
                self.expect_token(Token::CloseParen, "Expected ')' after expression", line, col); // Expect closing parenthesis
                self.expect_token(Token::Semicolon, "Expected ';' after print", line, col); // Expect semicolon
                Stmt::Print(expr) // Return Print statement
            }

            Token::If => {
                self.next();
                self.expect_token(Token::OpenParen, "Expected '(' after 'if'", line, col); // Expect opening parenthesis
                let condition = self.expression(); // Parse the condition
                self.expect_token(Token::CloseParen, "Expected ')' after condition", line, col); // Expect closing parenthesis
                let then_branch = Box::new(self.statement()); // Parse the then branch
                let else_branch = if self.current_token == Token::Else {
                    self.next();
                    Some(Box::new(self.statement())) // Parse the else branch
                } else {
                    None
                };
                Stmt::If { condition, then_branch, else_branch } // Return If statement
            }

            Token::While => {
                self.next();
                self.expect_token(Token::OpenParen, "Expected '(' after 'while'", line, col); // Expect opening parenthesis
                let condition = self.expression(); // Parse the condition
                self.expect_token(Token::CloseParen, "Expected ')' after condition", line, col); // Expect closing parenthesis
                let body = Box::new(self.statement()); // Parse the body
                Stmt::While { condition, body } // Return While statement
            }

            Token::OpenBrace => self.block(), // Parse a block statement

            Token::Enum => {
                self.next();
                self.expect_token(Token::OpenBrace, "Expected '{' after 'enum'", line, col); // Expect opening brace
                let mut value = 0;
                while self.current_token != Token::CloseBrace {
                    let name = self.expect_identifier("Expected identifier in enum", line, col); // Parse enum name
                    if self.current_token == Token::Assign {
                        self.next();
                        if let Token::Num(n) = self.current_token {
                            value = n;
                            self.next();
                        } else {
                            panic!("Expected number after '=' in enum");
                        }
                    }
                    self.vm.constants.insert(name.clone(), value); // Insert constant into VM constants
                    value += 1;
                    if self.current_token == Token::Comma {
                        self.next(); // Consume the comma if present
                    } else if self.current_token != Token::CloseBrace {
                        panic!("Expected ',' or '}}' in enum declaration");
                    }
                }
                self.expect_token(Token::CloseBrace, "Expected '}' after enum", line, col); // Expect closing brace
                self.expect_token(Token::Semicolon, "Expected ';' after enum", line, col); // Expect semicolon
                Stmt::Block(vec![]) // Return an empty block
            }

            _ => {
                let expr = self.expression(); // Parse expression statement
                self.expect_token(Token::Semicolon, "Expected ';' after expression", line, col); // Expect semicolon
                Stmt::ExprStmt(expr) // Return Expression statement
            }
        }
    }

    // Parse expressions and handle different precedence levels
    fn expression(&mut self) -> Expr {
        self.parse_ternary() // Start with ternary operator parsing
    }

    fn parse_ternary(&mut self) -> Expr {
        let condition = self.parse_assignment(); // Parse assignment expression
        if self.current_token == Token::QuestionMark { // If ternary operator found
            self.next();
            let then_branch = self.expression(); // Parse then branch
            self.expect_token(Token::Colon, "Expected ':' in ternary", 0, 0); // Expect colon
            let else_branch = self.expression(); // Parse else branch
            Expr::Ternary {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            }
        } else {
            condition
        }
    }

    /// Parses assignment expressions (variable assignments or array assignments).
    fn parse_assignment(&mut self) -> Expr {
        let lhs = self.parse_logic_or(); // Parse the left-hand side of the assignment
        if self.current_token == Token::Assign { // If the current token is an assignment operator
            self.next(); // Consume the assignment token
            let rhs = self.parse_assignment(); // Parse the right-hand side of the assignment
            match lhs {
                Expr::Variable(name) => Expr::BinaryOp { // Handle variable assignment
                    op: BinOp::Assign, // Assignment operation
                    left: Box::new(Expr::Variable(name)),
                    right: Box::new(rhs),
                },
                Expr::ArrayIndex(array, index) => Expr::BinaryOp { // Handle array assignment
                    op: BinOp::Assign,
                    left: Box::new(Expr::ArrayIndex(array, index)),
                    right: Box::new(rhs),
                },
                _ => panic!("Invalid assignment target"), // Error if the left-hand side is not a valid target
            }
        } else {
            lhs // If no assignment operator, return the left-hand side expression
        }
    }

    /// Parses logical OR expressions (using `||`).
    fn parse_logic_or(&mut self) -> Expr {
        let mut lhs = self.parse_logic_and(); // Parse logical AND expression first
        while self.current_token == Token::Or { // While we have a logical OR token
            self.next(); // Consume the OR token
            let rhs = self.parse_logic_and(); // Parse the right-hand side of the OR operation
            lhs = Expr::BinaryOp { // Build a binary operation for OR
                op: BinOp::Or,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        lhs // Return the result of the OR operation
    }

    /// Parses logical AND expressions (using `&&`).
    fn parse_logic_and(&mut self) -> Expr {
        let mut lhs = self.parse_bit_or(); // Parse bitwise OR first
        while self.current_token == Token::And { // While we have a logical AND token
            self.next(); // Consume the AND token
            let rhs = self.parse_bit_or(); // Parse the right-hand side of the AND operation
            lhs = Expr::BinaryOp { // Build a binary operation for AND
                op: BinOp::And,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        lhs // Return the result of the AND operation
    }

    /// Parses bitwise OR expressions (using `|`).
    fn parse_bit_or(&mut self) -> Expr {
        let mut lhs = self.parse_bit_xor(); // Parse bitwise XOR first
        while self.current_token == Token::BitOr { // While we have a bitwise OR token
            self.next(); // Consume the OR token
            let rhs = self.parse_bit_xor(); // Parse the right-hand side of the OR operation
            lhs = Expr::BinaryOp { // Build a binary operation for OR
                op: BinOp::BitOr,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        lhs // Return the result of the OR operation
    }

    /// Parses bitwise XOR expressions (using `^`).
    fn parse_bit_xor(&mut self) -> Expr {
        let mut lhs = self.parse_bit_and(); // Parse bitwise AND first
        while self.current_token == Token::BitXor { // While we have a bitwise XOR token
            self.next(); // Consume the XOR token
            let rhs = self.parse_bit_and(); // Parse the right-hand side of the XOR operation
            lhs = Expr::BinaryOp { // Build a binary operation for XOR
                op: BinOp::BitXor,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        lhs // Return the result of the XOR operation
    }

    /// Parses bitwise AND expressions (using `&`).
    fn parse_bit_and(&mut self) -> Expr {
        let mut lhs = self.parse_cmp(); // Parse comparison expressions first
        while self.current_token == Token::BitAnd { // While we have a bitwise AND token
            self.next(); // Consume the AND token
            let rhs = self.parse_cmp(); // Parse the right-hand side of the AND operation
            lhs = Expr::BinaryOp { // Build a binary operation for AND
                op: BinOp::BitAnd,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        lhs // Return the result of the AND operation
    }

    /// Parses comparison expressions (e.g., `==`, `!=`, `<`, `>`, `<=`, `>=`).
    fn parse_cmp(&mut self) -> Expr {
        let mut lhs = self.parse_shift(); // Parse shift operations first

        while matches!(self.current_token, Token::Equal | Token::NotEqual | Token::LessThan | Token::GreaterThan | Token::LessEqual | Token::GreaterEqual) {
            let op = match self.current_token {
                Token::Equal => BinOp::Equal,
                Token::NotEqual => BinOp::NotEqual,
                Token::LessThan => BinOp::LessThan,
                Token::GreaterThan => BinOp::GreaterThan,
                Token::LessEqual => BinOp::LessEqual,
                Token::GreaterEqual => BinOp::GreaterEqual,
                _ => unreachable!(),
            };
            self.next(); // Consume the comparison operator
            let rhs = self.parse_add_sub(); // Parse the right-hand side of the comparison
            lhs = Expr::BinaryOp { // Build a binary operation for comparison
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        lhs // Return the result of the comparison
    }

    /// Parses shift expressions (e.g., `<<`, `>>`).
    fn parse_shift(&mut self) -> Expr {
        let mut lhs = self.parse_add_sub(); // Parse addition and subtraction first
        while matches!(self.current_token, Token::Shl | Token::Shr) { // While we have shift tokens
            let op = match self.current_token {
                Token::Shl => BinOp::Shl,
                Token::Shr => BinOp::Shr,
                _ => unreachable!(),
            };
            self.next(); // Consume the shift token
            let rhs = self.parse_add_sub(); // Parse the right-hand side of the shift operation
            lhs = Expr::BinaryOp { // Build a binary operation for shift
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        lhs // Return the result of the shift operation
    }

    /// Parses addition and subtraction expressions (e.g., `+`, `-`).
    fn parse_add_sub(&mut self) -> Expr {
        let mut lhs = self.parse_mul_div(); // Parse multiplication and division first
        while matches!(self.current_token, Token::Add | Token::Sub) { // While we have addition or subtraction tokens
            let op = match self.current_token {
                Token::Add => BinOp::Add,
                Token::Sub => BinOp::Sub,
                _ => unreachable!(),
            };
            self.next(); // Consume the addition or subtraction token
            let rhs = self.parse_mul_div(); // Parse the right-hand side of the operation
            lhs = Expr::BinaryOp { // Build a binary operation for addition or subtraction
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        lhs // Return the result of the addition or subtraction
    }

    /// Parses multiplication, division, and modulus expressions (e.g., `*`, `/`, `%`).
    fn parse_mul_div(&mut self) -> Expr {
        let mut lhs = self.parse_unary(); // Parse unary operations first
        while matches!(self.current_token, Token::Mul | Token::Div | Token::Mod) { // While we have multiplication, division, or modulus tokens
            let op = match self.current_token {
                Token::Mul => BinOp::Mul,
                Token::Div => BinOp::Div,
                Token::Mod => BinOp::Mod,
                _ => unreachable!(),
            };
            self.next(); // Consume the multiplication, division, or modulus token
            let rhs = self.parse_unary(); // Parse the right-hand side of the operation
            lhs = Expr::BinaryOp { // Build a binary operation for multiplication, division, or modulus
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        lhs // Return the result of the multiplication, division, or modulus
    }

    /// Parses unary operations (e.g., negation, dereference, address-of).
    fn parse_unary(&mut self) -> Expr {
        let expr = match self.current_token {
            Token::Not => {
                self.next();
                let expr = self.parse_unary(); // Parse the right-hand side of the NOT operation
                Expr::UnaryOp { op: UnOp::Not, expr: Box::new(expr) } // Return a NOT operation
            }
            Token::AddressOf => {
                self.next();
                let expr = self.parse_unary(); // Parse the right-hand side of the address-of operation
                Expr::AddressOf(Box::new(expr)) // Return an AddressOf operation
            }
            Token::Deref => {
                self.next();
                let expr = self.parse_unary(); // Parse the right-hand side of the dereference operation
                Expr::Deref(Box::new(expr)) // Return a Deref operation
            }
            Token::PlusPlus => {
                self.next();
                let expr = self.parse_unary(); // Parse the right-hand side of the pre-increment operation
                Expr::PreInc(Box::new(expr)) // Return a pre-increment operation
            }
            Token::MinusMinus => {
                self.next();
                let expr = self.parse_unary(); // Parse the right-hand side of the pre-decrement operation
                Expr::PreDec(Box::new(expr)) // Return a pre-decrement operation
            }
            Token::BitAnd => {  // ✅ For bitwise AND
                self.next();
                let expr = self.parse_unary(); // Parse the right-hand side of the bitwise AND operation
                Expr::AddressOf(Box::new(expr)) // Return an AddressOf operation
            }
            Token::Mul => {     // ✅ For dereference
                self.next();
                let expr = self.parse_unary(); // Parse the right-hand side of the dereference operation
                Expr::Deref(Box::new(expr)) // Return a Deref operation
            }
            _ => self.parse_primary(), // Parse primary expression if no unary operator
        };
        self.parse_postfix(expr) // Handle postfix operations like increment and decrement
    }

    /// Handles postfix operations (e.g., `++`, `--`).
    fn parse_postfix(&mut self, mut expr: Expr) -> Expr {
        loop {
            match self.current_token {
                Token::PlusPlus => {
                    self.next();
                    expr = Expr::PostInc(Box::new(expr)); // Post-increment operation
                }
                Token::MinusMinus => {
                    self.next();
                    expr = Expr::PostDec(Box::new(expr)); // Post-decrement operation
                }
                _ => break, // Exit loop if no more postfix operators
            }
        }
        expr // Return the final expression with postfix operations applied
    }

    /// Parses primary expressions (e.g., numbers, strings, identifiers, etc.)
    fn parse_primary(&mut self) -> Expr {
        let (line, col) = self.lexer.get_position(); // Get the position of the current token
        match &self.current_token {
            Token::Num(n) => { let val = *n; self.next(); Expr::Number(val) } // Parse number literal
            Token::True => { self.next(); Expr::Boolean(true) } // Parse boolean true
            Token::False => { self.next(); Expr::Boolean(false) } // Parse boolean false
            Token::Char(c) => { let ch = *c; self.next(); Expr::Char(ch) } // Parse character literal
            Token::StringLiteral(s) => { let val = s.clone(); self.next(); Expr::StringLiteral(val) } // Parse string literal
    
            Token::Sizeof => {
                self.next();
                self.expect_token(Token::OpenParen, "Expected '(' after sizeof", line, col); // Expect '('
                let typ = self.parse_type().unwrap_or(Type::Int); // Parse the type after sizeof
                self.expect_token(Token::CloseParen, "Expected ')' after type", line, col); // Expect ')'
                Expr::SizeOf(typ) // Return SizeOf expression
            }
    
            Token::OpenBrace => {
                self.next();
                let mut elements = Vec::new(); // Initialize a vector for array elements
                while self.current_token != Token::CloseBrace { // Parse array elements until we encounter a closing brace
                    elements.push(self.expression()); // Parse each element in the array
                    if self.current_token == Token::Comma {
                        self.next(); // Consume the comma if present
                    } else {
                        break; // Exit loop if no more elements
                    }
                }
                self.expect_token(Token::CloseBrace, "Expected '}' after array literal", line, col); // Expect closing brace
                Expr::ArrayLiteral(elements) // Return an ArrayLiteral expression
            }
    
            // ✅ Support array literals like [1, 2, 3]
            Token::OpenBracket => {
                self.next();
                let mut elements = Vec::new(); // Initialize a vector for array elements
                while self.current_token != Token::CloseBracket { // Parse array elements until we encounter a closing bracket
                    elements.push(self.expression()); // Parse each element in the array
                    if self.current_token == Token::Comma {
                        self.next(); // Consume the comma if present
                    } else {
                        break; // Exit loop if no more elements
                    }
                }
                self.expect_token(Token::CloseBracket, "Expected ']' after array literal", line, col); // Expect closing bracket
                Expr::ArrayLiteral(elements) // Return an ArrayLiteral expression
            }
    
            Token::Identifier(name) => {
                let id = name.clone(); // Parse the identifier
                self.next();
    
                if self.current_token == Token::OpenParen { // If the next token is '(', it’s a function call
                    self.next();
                    let mut args = Vec::new(); // Initialize a vector for function arguments
                    while self.current_token != Token::CloseParen { // Parse function arguments
                        args.push(self.expression()); // Parse each argument
                        if self.current_token == Token::Comma {
                            self.next(); // Consume the comma if present
                        }
                    }
                    self.expect_token(Token::CloseParen, "Expected ')' after arguments", line, col); // Expect closing parenthesis
                    Expr::FunctionCall { name: id, args } // Return a FunctionCall expression
                }
                else if self.current_token == Token::OpenBracket { // If the next token is '[', it’s an array index
                    self.next();
                    let index_expr = self.expression(); // Parse the index expression
                    self.expect_token(Token::CloseBracket, "Expected ']' after array index", line, col); // Expect closing bracket
                    Expr::ArrayIndex(Box::new(Expr::Variable(id)), Box::new(index_expr)) // Return an ArrayIndex expression
                }
                else {
                    Expr::Variable(id) // Return a Variable expression
                }
            }
    
            Token::OpenParen => {
                self.next();
                let is_type = match &self.current_token {
                    Token::Identifier(tn) => matches!(tn.as_str(), "int" | "char" | "bool" | "str" | "void"), // Check if it’s a type
                    Token::Mul => true, // Handle pointer types
                    _ => false,
                };
                if is_type {
                    let typ = self.parse_type().unwrap(); // Parse type inside parentheses
                    self.expect_token(Token::CloseParen, "Expected ')' after type", line, col); // Expect closing parenthesis
                    let expr = self.parse_unary(); // Parse the unary expression
                    Expr::Cast(typ, Box::new(expr)) // Return a Cast expression
                } else {
                    let expr = self.expression(); // Parse the regular expression
                    self.expect_token(Token::CloseParen, "Expected ')' after expression", line, col); // Expect closing parenthesis
                    expr // Return the parsed expression
                }
            }
    
            _ => panic!("Unexpected token at line {}, column {}: {:?}", line, col, self.current_token), // Handle unexpected tokens
        }
    }
    
    /// Parses a type (e.g., `int`, `char`, `void`).
    fn parse_type(&mut self) -> Option<Type> {
        let mut base = match self.current_token {
            Token::Identifier(ref name) => match name.as_str() {
                "int" => { self.next(); Type::Int } // Parse int type
                "char" => { self.next(); Type::Char } // Parse char type
                "bool" => { self.next(); Type::Char } // Parse bool type (treated as char for now)
                "str" => { self.next(); Type::Pointer(Box::new(Type::Char)) } // Parse string type (pointer to char)
                "void" => { self.next(); Type::Void } // Parse void type
                _ => panic!("Unknown type '{}'", name), // Handle unknown types
            },
            Token::Mul => {
                self.next();
                return self.parse_type().map(|t| Type::Pointer(Box::new(t))); // Handle pointer type
            }
            _ => return None, // If no type is found, return None
        };
    
        while self.current_token == Token::OpenBracket { // Handle array types (e.g., `int[]`)
            self.next();
            if let Token::Num(n) = self.current_token {
                self.next();
                self.expect_token(Token::CloseBracket, "Expected ']' after array size", 0, 0); // Expect closing bracket
                base = Type::Array(Box::new(base), n as usize); // Build array type
            } else {
                panic!("Expected array size inside brackets"); // Error if no array size is specified
            }
        }
    
        Some(base) // Return the parsed type
    }
    

     /// Parses a block of statements (enclosed in `{}`).
     fn block(&mut self) -> Stmt {
        self.expect_token(Token::OpenBrace, "Expected '{' to start block", 0, 0); // Expect opening brace
        let mut stmts = Vec::new(); // Initialize an empty vector for statements
        while self.current_token != Token::CloseBrace { // Parse statements until we encounter closing brace
            let stmt = self.statement(); // Parse each statement
            stmts.push(stmt); // Add the statement to the list
        }
        self.next(); // Consume closing brace
        Stmt::Block(stmts) // Return the block of statements
    }

    /// Expects a specific token and advances the parser, or panics with an error message if the token doesn't match.
    fn expect_token(&mut self, expected: Token, msg: &str, line: usize, col: usize) {
        if self.current_token != expected {
            panic!("{} at line {}, column {}", msg, line, col); // If the token doesn't match, panic
        }
        self.next(); // Consume the expected token
    }

    /// Expects an identifier and advances the parser, or panics with an error message if the token isn't an identifier.
    fn expect_identifier(&mut self, msg: &str, line: usize, col: usize) -> String {
        if let Token::Identifier(n) = &self.current_token {
            let name = n.clone();
            self.next();
            name // Return the identifier
        } else {
            panic!("{} at line {}, column {}", msg, line, col); // Error if the token is not an identifier
        }
    }
}