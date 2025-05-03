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
                    self.type_map.insert(name.clone(), var_type.clone()); // fix here
                    self.consume_semicolon();
                    return Stmt::Let { name, value, var_type: Some(var_type) }; // now safe
                    
                }
            }
        }

        match &self.current_token {
            Token::Return => {
                self.next();
                if self.current_token == Token::Semicolon {
                    self.next();
                    Stmt::Return(Expr::Number(0))
                } else {
                    let expr = self.expression();
                    self.consume_semicolon();
                    Stmt::Return(expr)
                }
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
                self.consume_semicolon();
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
                self.consume_semicolon();
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
                let enum_name = format!("__anon_enum_{}", line);
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
                            panic!("Expected number after '=' in enum declaration");
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
                self.consume_semicolon();
                Stmt::Block(vec![])
            }

            _ => {
                let expr = self.expression();
                self.consume_semicolon();
                Stmt::ExprStmt(expr)
            }
        }
    }

    fn expression(&mut self) -> Expr {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Expr {
        let lhs = self.parse_logic_or();
        if self.current_token == Token::Assign {
            self.next();
            let rhs = self.parse_assignment();
            if let Expr::Variable(name) = lhs {
                Expr::BinaryOp {
                    op: BinOp::Assign,
                    left: Box::new(Expr::Variable(name)),
                    right: Box::new(rhs),
                }
            } else {
                panic!("Invalid assignment target");
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
        let mut lhs = self.parse_cmp();
        while self.current_token == Token::And {
            self.next();
            let rhs = self.parse_cmp();
            lhs = Expr::BinaryOp { op: BinOp::And, left: Box::new(lhs), right: Box::new(rhs) };
        }
        lhs
    }

    fn parse_cmp(&mut self) -> Expr {
        let mut lhs = self.parse_add_sub();
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
        while matches!(self.current_token, Token::Mul | Token::Div) {
            let op = match self.current_token {
                Token::Mul => BinOp::Mul,
                Token::Div => BinOp::Div,
                _ => unreachable!(),
            };
            self.next();
            let rhs = self.parse_unary();
            lhs = Expr::BinaryOp { op, left: Box::new(lhs), right: Box::new(rhs) };
        }
        lhs
    }

    fn parse_unary(&mut self) -> Expr {
        match self.current_token {
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
            _ => self.parse_primary(),
        }
    }
    

fn parse_primary(&mut self) -> Expr {
let (line, col) = self.lexer.get_position();
match &self.current_token {
Token::Num(n) => {
    let val = *n;
    self.next();
    Expr::Number(val)
}
Token::True => {
    self.next();
    Expr::Boolean(true)
}
Token::False => {
    self.next();
    Expr::Boolean(false)
}
Token::Char(c) => {
    let ch = *c;
    self.next();
    Expr::Char(ch)
}
Token::StringLiteral(s) => {
    let val = s.clone();
    self.next();
    Expr::StringLiteral(val)
}
Token::Sizeof => {
    self.next();
    self.expect_token(Token::OpenParen, "Expected '(' after sizeof", line, col);
    let typ = self.parse_type().unwrap_or(Type::Int);
    self.expect_token(Token::CloseParen, "Expected ')' after type", line, col);
    Expr::SizeOf(typ)
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
    } else {
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
match self.current_token {
Token::Identifier(ref name) => {
    match name.as_str() {
        "int" => {
            self.next();
            Some(Type::Int)
        }
        "char" => {
            self.next();
            Some(Type::Char)
        }
        "bool" => {
            self.next();
            Some(Type::Char)
        }
        "str" => {
            self.next();
            Some(Type::Pointer(Box::new(Type::Char)))
        }
        "void" => {
            self.next();
            Some(Type::Void)
        }
        other => panic!("Unknown type '{}'", other),
    }
}
Token::Mul => {
    self.next();
    self.parse_type().map(|t| Type::Pointer(Box::new(t)))
}
_ => None,
}
}

fn block(&mut self) -> Stmt {
self.expect_token(Token::OpenBrace, "Expected '{' to start block", 0, 0);
let mut stmts = Vec::new();
while self.current_token != Token::CloseBrace {
stmts.push(self.statement());
}
self.next();
Stmt::Block(stmts)
}

fn consume_semicolon(&mut self) {
if self.current_token == Token::Semicolon {
self.next();
} else if self.current_token != Token::CloseBrace && self.current_token != Token::Eof {
let (line, col) = self.lexer.get_position();
panic!("Expected ';' at line {}, column {}", line, col);
}
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
