use crate::lexer::{Lexer, Token};
use crate::ast::{Expr, Stmt};
use crate::vm::Vm;

pub struct Parser<'a> {
    lexer: Lexer,
    current_token: Token,
    vm: &'a mut Vm,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer, vm: &'a mut Vm) -> Self {
        let mut parser = Self {
            lexer,
            current_token: Token::Eof, // Placeholder
            vm,
        };
        parser.next(); // Load the first token
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
        match &self.current_token {
            Token::Return => {
                self.next(); // consume 'return'
                let expr = self.expression();
                if self.current_token == Token::Semicolon {
                    self.next(); // consume ';'
                } else {
                    panic!("Expected ';' after return");
                }
                Stmt::Return(expr)
            }
            Token::If => {
                self.next(); // consume 'if'
                if self.current_token != Token::OpenParen {
                    panic!("Expected '(' after if");
                }
                self.next();
                let condition = self.expression();
                if self.current_token != Token::CloseParen {
                    panic!("Expected ')' after condition");
                }
                self.next();
                let then_branch = Box::new(self.block()); // block!
                let else_branch = if self.current_token == Token::Else {
                    self.next();
                    Some(Box::new(self.block())) // block!
                } else {
                    None
                };
                Stmt::If { condition, then_branch, else_branch }
            }
            Token::While => {
                self.next(); // consume 'while'
                if self.current_token != Token::OpenParen {
                    panic!("Expected '(' after while");
                }
                self.next();
                let condition = self.expression();
                if self.current_token != Token::CloseParen {
                    panic!("Expected ')' after condition");
                }
                self.next();
                let body = Box::new(self.block()); // block!
                Stmt::While { condition, body }
            }
            Token::Let => {
                self.next(); // consume 'let'
                let var_name = if let Token::Identifier(name) = &self.current_token {
                    name.clone()
                } else {
                    panic!("Expected identifier after 'let'");
                };
                self.next(); // consume identifier
                if self.current_token != Token::Assign {
                    panic!("Expected '=' after identifier");
                }
                self.next(); // consume '='
                let value = self.expression();
                if self.current_token == Token::Semicolon {
                    self.next(); // consume ';'
                } else {
                    panic!("Expected ';' after let");
                }
                Stmt::Let { name: var_name, value }
            }
            Token::Identifier(name) => {
                let var_name = name.clone();
                self.next(); // consume identifier
                if self.current_token != Token::Assign {
                    panic!("Expected '=' after identifier");
                }
                self.next(); // consume '='
                let value = self.expression();
                if self.current_token == Token::Semicolon {
                    self.next(); // consume ';'
                } else {
                    panic!("Expected ';' after assignment");
                }
                Stmt::Assign { name: var_name, value }
            }
            _ => {
                panic!("Unexpected token: {:?}", self.current_token);
            }
        }
    }
    
    

    fn expression(&mut self) -> Expr {
        let mut node = self.primary();

        loop {
            match self.current_token {
                Token::Add => {
                    self.next();
                    node = Expr::BinaryOp {
                        op: crate::ast::BinOp::Add,
                        left: Box::new(node),
                        right: Box::new(self.primary()),
                    };
                }
                Token::Sub => {
                    self.next();
                    node = Expr::BinaryOp {
                        op: crate::ast::BinOp::Sub,
                        left: Box::new(node),
                        right: Box::new(self.primary()),
                    };
                }
                Token::Mul => {
                    self.next();
                    node = Expr::BinaryOp {
                        op: crate::ast::BinOp::Mul,
                        left: Box::new(node),
                        right: Box::new(self.primary()),
                    };
                }
                Token::Div => {
                    self.next();
                    node = Expr::BinaryOp {
                        op: crate::ast::BinOp::Div,
                        left: Box::new(node),
                        right: Box::new(self.primary()),
                    };
                }
                Token::Equal => {
                    self.next();
                    node = Expr::BinaryOp {
                        op: crate::ast::BinOp::Equal,
                        left: Box::new(node),
                        right: Box::new(self.primary()),
                    };
                }
                Token::NotEqual => {
                    self.next();
                    node = Expr::BinaryOp {
                        op: crate::ast::BinOp::NotEqual,
                        left: Box::new(node),
                        right: Box::new(self.primary()),
                    };
                }
                Token::LessThan => {
                    self.next();
                    node = Expr::BinaryOp {
                        op: crate::ast::BinOp::LessThan,
                        left: Box::new(node),
                        right: Box::new(self.primary()),
                    };
                }
                Token::GreaterThan => {
                    self.next();
                    node = Expr::BinaryOp {
                        op: crate::ast::BinOp::GreaterThan,
                        left: Box::new(node),
                        right: Box::new(self.primary()),
                    };
                }
                _ => break,
            }
        }

        node
    }

    fn primary(&mut self) -> Expr {
        match &self.current_token {
            Token::Num(n) => {
                let val = *n;
                self.next();
                Expr::Number(val)
            }
            Token::Identifier(name) => {
                let var_name = name.clone();
                self.next();
                Expr::Variable(var_name)
            }
            Token::OpenParen => {
                self.next();
                let expr = self.expression();
                if self.current_token == Token::CloseParen {
                    self.next();
                    expr
                } else {
                    panic!("Expected closing parenthesis");
                }
            }
            _ => {
                panic!("Unexpected token in primary: {:?}", self.current_token);
            }
        }
    }

    fn block(&mut self) -> Stmt {
        if self.current_token != Token::OpenBrace {
            return self.statement(); // If no `{`, just parse single statement
        }
    
        self.next(); // consume '{'
        let mut statements = Vec::new();
    
        while self.current_token != Token::CloseBrace {
            statements.push(self.statement());
        }
    
        self.next(); // consume '}'
    
        Stmt::Block(statements)
    }
}
