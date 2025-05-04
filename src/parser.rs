use crate::ast::{Expr, Stmt, BinOp, UnOp, Type};
use crate::lexer::{Lexer, Token};
use crate::vm::Vm;
use std::collections::HashMap;

pub struct Parser<'a> {
    lexer: Lexer,
    current_token: Token,
    vm: &'a mut Vm,
    type_map: HashMap<String, Type>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer, vm: &'a mut Vm) -> Self {
        let mut parser = Self {
            lexer,
            current_token: Token::Eof,
            vm,
            type_map: HashMap::new(),
        };
        parser.next();
        parser
    }

    pub fn next(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while self.current_token != Token::Eof {
            statements.push(self.statement());
        }
        statements
    }

    fn statement(&mut self) -> Stmt {
        let (line, col) = self.lexer.get_position();

        // Function or typed variable declaration
        if let Token::Identifier(ref type_name) = self.current_token {
            if matches!(type_name.as_str(), "int" | "char" | "bool" | "str" | "void") {
                let var_type = self.parse_type().unwrap();
                let (name_line, name_col) = self.lexer.get_position();
                let name = self.expect_identifier("Expected name after type", name_line, name_col);

                if self.current_token == Token::OpenParen {
                    self.next();
                    let mut params = Vec::new();
                    while self.current_token != Token::CloseParen {
                        let param_name = self.expect_identifier("Expected parameter name", line, col);
                        params.push(param_name);
                        if self.current_token == Token::Comma {
                            self.next();
                        } else if self.current_token != Token::CloseParen {
                            panic!("Expected ',' or ')' in parameter list at line {}, column {}", line, col);
                        }
                    }
                    self.expect_token(Token::CloseParen, "Expected ')' after parameters", line, col);
                    let body = Box::new(self.block());
                    return Stmt::Function {
                        name,
                        params,
                        body,
                        return_type: Some(var_type),
                    };
                } else {
                    self.expect_token(Token::Assign, "Expected '=' after variable name", line, col);
                    let value = self.expression();
                    self.type_map.insert(name.clone(), var_type.clone());
                    self.expect_token(Token::Semicolon, "Expected ';' after variable declaration", line, col);
                    return Stmt::Let { name, value, var_type: Some(var_type) };
                }
            }
        }

        match &self.current_token {
            Token::Return => {
                self.next();
                let expr = if matches!(self.current_token, Token::Semicolon | Token::CloseBrace) {
                    Expr::Number(0)
                } else {
                    self.expression()
                };
                if self.current_token == Token::Semicolon {
                    self.next();
                }
                Stmt::Return(expr)
            }

            Token::Let => {
                self.next();
                let mut decls = Vec::new();
                loop {
                    let name = self.expect_identifier("Expected identifier after 'let'", line, col);
                    let var_type = if self.current_token == Token::Colon {
                        self.next();
                        self.parse_type().unwrap_or(Type::Int)
                    } else {
                        Type::Int
                    };
                    self.expect_token(Token::Assign, "Expected '=' after identifier", line, col);
                    let value = self.expression();
                    self.type_map.insert(name.clone(), var_type.clone());
                    decls.push(Stmt::Let { name, value, var_type: Some(var_type) });
                    if self.current_token == Token::Comma {
                        self.next();
                    } else {
                        break;
                    }
                }
                self.expect_token(Token::Semicolon, "Expected ';' after let", line, col);
                if decls.len() == 1 {
                    decls.pop().unwrap()
                } else {
                    Stmt::Block(decls)
                }
            }

            Token::Print => {
                self.next();
                self.expect_token(Token::OpenParen, "Expected '(' after 'print'", line, col);
                let expr = self.expression();
                self.expect_token(Token::CloseParen, "Expected ')' after expression", line, col);
                self.expect_token(Token::Semicolon, "Expected ';' after print", line, col);
                Stmt::Print(expr)
            }

            Token::If => {
                self.next();
                self.expect_token(Token::OpenParen, "Expected '(' after 'if'", line, col);
                let condition = self.expression();
                self.expect_token(Token::CloseParen, "Expected ')' after condition", line, col);
                let then_branch = Box::new(self.statement());
                let else_branch = if self.current_token == Token::Else {
                    self.next();
                    Some(Box::new(self.statement()))
                } else {
                    None
                };
                Stmt::If { condition, then_branch, else_branch }
            }

            Token::While => {
                self.next();
                self.expect_token(Token::OpenParen, "Expected '(' after 'while'", line, col);
                let condition = self.expression();
                self.expect_token(Token::CloseParen, "Expected ')' after condition", line, col);
                let body = Box::new(self.statement());
                Stmt::While { condition, body }
            }

            Token::OpenBrace => self.block(),

            Token::Enum => {
                self.next();
                self.expect_token(Token::OpenBrace, "Expected '{' after 'enum'", line, col);
                let mut value = 0;
                while self.current_token != Token::CloseBrace {
                    let name = self.expect_identifier("Expected identifier in enum", line, col);
                    if self.current_token == Token::Assign {
                        self.next();
                        if let Token::Num(n) = self.current_token {
                            value = n;
                            self.next();
                        } else {
                            panic!("Expected number after '=' in enum");
                        }
                    }
                    self.vm.constants.insert(name.clone(), value);
                    value += 1;
                    if self.current_token == Token::Comma {
                        self.next();
                    } else if self.current_token != Token::CloseBrace {
                        panic!("Expected ',' or '}}' in enum declaration");
                    }
                }
                self.expect_token(Token::CloseBrace, "Expected '}' after enum", line, col);
                self.expect_token(Token::Semicolon, "Expected ';' after enum", line, col);
                Stmt::Block(vec![])
            }

            _ => {
                let expr = self.expression();
                self.expect_token(Token::Semicolon, "Expected ';' after expression", line, col);
                Stmt::ExprStmt(expr)
            }
        }
    }

    fn expression(&mut self) -> Expr {
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Expr {
        let condition = self.parse_assignment();
        if self.current_token == Token::QuestionMark {
            self.next();
            let then_branch = self.expression();
            self.expect_token(Token::Colon, "Expected ':' in ternary", 0, 0);
            let else_branch = self.expression();
            Expr::Ternary {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            }
        } else {
            condition
        }
    }

    fn parse_assignment(&mut self) -> Expr {
        let lhs = self.parse_logic_or();
        if self.current_token == Token::Assign {
            self.next();
            let rhs = self.parse_assignment();
            match lhs {
                Expr::Variable(name) => Expr::BinaryOp {
                    op: BinOp::Assign,
                    left: Box::new(Expr::Variable(name)),
                    right: Box::new(rhs),
                },
                Expr::ArrayIndex(array, index) => Expr::BinaryOp {
                    op: BinOp::Assign,
                    left: Box::new(Expr::ArrayIndex(array, index)),
                    right: Box::new(rhs),
                },
                _ => panic!("Invalid assignment target"),
            }
        } else {
            lhs
        }
    }
    

    fn parse_logic_or(&mut self) -> Expr {
        let mut lhs = self.parse_logic_and();
        while self.current_token == Token::Or {
            self.next();
            let rhs = self.parse_logic_and();
            lhs = Expr::BinaryOp { op: BinOp::Or, left: Box::new(lhs), right: Box::new(rhs) };
        }
        lhs
    }

    fn parse_logic_and(&mut self) -> Expr {
        let mut lhs = self.parse_bit_or();
        while self.current_token == Token::And {
            self.next();
            let rhs = self.parse_bit_or();
            lhs = Expr::BinaryOp { op: BinOp::And, left: Box::new(lhs), right: Box::new(rhs) };
        }
        lhs
    }

    fn parse_bit_or(&mut self) -> Expr {
        let mut lhs = self.parse_bit_xor();
        while self.current_token == Token::BitOr {
            self.next();
            let rhs = self.parse_bit_xor();
            lhs = Expr::BinaryOp { op: BinOp::BitOr, left: Box::new(lhs), right: Box::new(rhs) };
        }
        lhs
    }
    
    fn parse_bit_xor(&mut self) -> Expr {
        let mut lhs = self.parse_bit_and();
        while self.current_token == Token::BitXor {
            self.next();
            let rhs = self.parse_bit_and();
            lhs = Expr::BinaryOp { op: BinOp::BitXor, left: Box::new(lhs), right: Box::new(rhs) };
        }
        lhs
    }
    
    fn parse_bit_and(&mut self) -> Expr {
        let mut lhs = self.parse_cmp();
        while self.current_token == Token::BitAnd {
            self.next();
            let rhs = self.parse_cmp();
            lhs = Expr::BinaryOp { op: BinOp::BitAnd, left: Box::new(lhs), right: Box::new(rhs) };
        }
        lhs
    }
    
    

    fn parse_cmp(&mut self) -> Expr {
        let mut lhs = self.parse_shift();

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
            self.next();
            let rhs = self.parse_add_sub();
            lhs = Expr::BinaryOp { op, left: Box::new(lhs), right: Box::new(rhs) };
        }
        lhs
    }

    fn parse_shift(&mut self) -> Expr {
        let mut lhs = self.parse_add_sub();
        while matches!(self.current_token, Token::Shl | Token::Shr) {
            let op = match self.current_token {
                Token::Shl => BinOp::Shl,
                Token::Shr => BinOp::Shr,
                _ => unreachable!(),
            };
            self.next();
            let rhs = self.parse_add_sub();
            lhs = Expr::BinaryOp { op, left: Box::new(lhs), right: Box::new(rhs) };
        }
        lhs
    }
    

    fn parse_add_sub(&mut self) -> Expr {
        let mut lhs = self.parse_mul_div();
        while matches!(self.current_token, Token::Add | Token::Sub) {
            let op = match self.current_token {
                Token::Add => BinOp::Add,
                Token::Sub => BinOp::Sub,
                _ => unreachable!(),
            };
            self.next();
            let rhs = self.parse_mul_div();
            lhs = Expr::BinaryOp { op, left: Box::new(lhs), right: Box::new(rhs) };
        }
        lhs
    }

    fn parse_mul_div(&mut self) -> Expr {
        let mut lhs = self.parse_unary();
        while matches!(self.current_token, Token::Mul | Token::Div | Token::Mod) {
            let op = match self.current_token {
                Token::Mul => BinOp::Mul,
                Token::Div => BinOp::Div,
                Token::Mod => BinOp::Mod,
                _ => unreachable!(),
            };
            self.next();
            let rhs = self.parse_unary();
            lhs = Expr::BinaryOp { op, left: Box::new(lhs), right: Box::new(rhs) };
        }
        lhs
    }
    

    fn parse_unary(&mut self) -> Expr {
        let expr = match self.current_token {
            Token::Not => {
                self.next();
                let expr = self.parse_unary();
                Expr::UnaryOp { op: UnOp::Not, expr: Box::new(expr) }
            }
            Token::AddressOf => {
                self.next();
                let expr = self.parse_unary();
                Expr::AddressOf(Box::new(expr))
            }
            Token::Deref => {
                self.next();
                let expr = self.parse_unary();
                Expr::Deref(Box::new(expr))
            }
            Token::PlusPlus => {
                self.next();
                let expr = self.parse_unary();
                Expr::PreInc(Box::new(expr))
            }
            Token::MinusMinus => {
                self.next();
                let expr = self.parse_unary();
                Expr::PreDec(Box::new(expr))
            }
            Token::BitAnd => {  // ✅ Add this
                self.next();
                let expr = self.parse_unary();
                Expr::AddressOf(Box::new(expr))
            }
            Token::Mul => {     // ✅ For dereference
                self.next();
                let expr = self.parse_unary();
                Expr::Deref(Box::new(expr))
            }
            _ => self.parse_primary(),
        };
        self.parse_postfix(expr)
    }

    fn parse_postfix(&mut self, mut expr: Expr) -> Expr {
        loop {
            match self.current_token {
                Token::PlusPlus => {
                    self.next();
                    expr = Expr::PostInc(Box::new(expr));
                }
                Token::MinusMinus => {
                    self.next();
                    expr = Expr::PostDec(Box::new(expr));
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> Expr {
        let (line, col) = self.lexer.get_position();
        match &self.current_token {
            Token::Num(n) => { let val = *n; self.next(); Expr::Number(val) }
            Token::True => { self.next(); Expr::Boolean(true) }
            Token::False => { self.next(); Expr::Boolean(false) }
            Token::Char(c) => { let ch = *c; self.next(); Expr::Char(ch) }
            Token::StringLiteral(s) => { let val = s.clone(); self.next(); Expr::StringLiteral(val) }
    
            Token::Sizeof => {
                self.next();
                self.expect_token(Token::OpenParen, "Expected '(' after sizeof", line, col);
                let typ = self.parse_type().unwrap_or(Type::Int);
                self.expect_token(Token::CloseParen, "Expected ')' after type", line, col);
                Expr::SizeOf(typ)
            }
    
            Token::OpenBrace => {
                self.next();
                let mut elements = Vec::new();
                while self.current_token != Token::CloseBrace {
                    elements.push(self.expression());
                    if self.current_token == Token::Comma {
                        self.next();
                    } else {
                        break;
                    }
                }
                self.expect_token(Token::CloseBrace, "Expected '}' after array literal", line, col);
                Expr::ArrayLiteral(elements)
            }
    
            // ✅ Support array literals like [1, 2, 3]
            Token::OpenBracket => {
                self.next();
                let mut elements = Vec::new();
                while self.current_token != Token::CloseBracket {
                    elements.push(self.expression());
                    if self.current_token == Token::Comma {
                        self.next();
                    } else {
                        break;
                    }
                }
                self.expect_token(Token::CloseBracket, "Expected ']' after array literal", line, col);
                Expr::ArrayLiteral(elements)
            }
    
            Token::Identifier(name) => {
                let id = name.clone();
                self.next();
    
                if self.current_token == Token::OpenParen {
                    self.next();
                    let mut args = Vec::new();
                    while self.current_token != Token::CloseParen {
                        args.push(self.expression());
                        if self.current_token == Token::Comma {
                            self.next();
                        }
                    }
                    self.expect_token(Token::CloseParen, "Expected ')' after arguments", line, col);
                    Expr::FunctionCall { name: id, args }
                }
                else if self.current_token == Token::OpenBracket {
                    self.next();
                    let index_expr = self.expression();
                    self.expect_token(Token::CloseBracket, "Expected ']' after array index", line, col);
                    Expr::ArrayIndex(Box::new(Expr::Variable(id)), Box::new(index_expr))
                }
                else {
                    Expr::Variable(id)
                }
            }
    
            Token::OpenParen => {
                self.next();
                let is_type = match &self.current_token {
                    Token::Identifier(tn) => matches!(tn.as_str(), "int" | "char" | "bool" | "str" | "void"),
                    Token::Mul => true,
                    _ => false,
                };
                if is_type {
                    let typ = self.parse_type().unwrap();
                    self.expect_token(Token::CloseParen, "Expected ')' after type", line, col);
                    let expr = self.parse_unary();
                    Expr::Cast(typ, Box::new(expr))
                } else {
                    let expr = self.expression();
                    self.expect_token(Token::CloseParen, "Expected ')' after expression", line, col);
                    expr
                }
            }
    
            _ => panic!("Unexpected token at line {}, column {}: {:?}", line, col, self.current_token),
        }
    }
    
    fn parse_type(&mut self) -> Option<Type> {
        let mut base = match self.current_token {
            Token::Identifier(ref name) => match name.as_str() {
                "int" => { self.next(); Type::Int }
                "char" => { self.next(); Type::Char }
                "bool" => { self.next(); Type::Char }
                "str" => { self.next(); Type::Pointer(Box::new(Type::Char)) }
                "void" => { self.next(); Type::Void }
                _ => panic!("Unknown type '{}'", name),
            },
            Token::Mul => {
                self.next();
                return self.parse_type().map(|t| Type::Pointer(Box::new(t)));
            }
            _ => return None,
        };
    
        while self.current_token == Token::OpenBracket {
            self.next();
            if let Token::Num(n) = self.current_token {
                self.next();
                self.expect_token(Token::CloseBracket, "Expected ']' after array size", 0, 0);
                base = Type::Array(Box::new(base), n as usize);
            } else {
                panic!("Expected array size inside brackets");
            }
        }
    
        Some(base)
    }
    

    fn block(&mut self) -> Stmt {
        self.expect_token(Token::OpenBrace, "Expected '{' to start block", 0, 0);
        let mut stmts = Vec::new();
        while self.current_token != Token::CloseBrace {
            let stmt = self.statement();
            stmts.push(stmt);
        }
        self.next(); // consume '}'
        Stmt::Block(stmts)
    }

    fn expect_token(&mut self, expected: Token, msg: &str, line: usize, col: usize) {
        if self.current_token != expected {
            panic!("{} at line {}, column {}", msg, line, col);
        }
        self.next();
    }

    fn expect_identifier(&mut self, msg: &str, line: usize, col: usize) -> String {
        if let Token::Identifier(n) = &self.current_token {
            let name = n.clone();
            self.next();
            name
        } else {
            panic!("{} at line {}, column {}", msg, line, col);
        }
    }
}
