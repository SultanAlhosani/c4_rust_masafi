use crate::lexer::{Lexer, Token};
use crate::ast::{Stmt, Expr, BinOp};
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
            current_token: Token::Eof,
            vm,
        };
        parser.next(); // Load the first token
        parser
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
                self.next();
                let expr = self.expression();
                if self.current_token == Token::Semicolon {
                    self.next();
                } else {
                    panic!("Expected ';' after return value");
                }
                Stmt::Return(expr)
            }
            Token::If => {
                self.next();
                if self.current_token != Token::OpenParen {
                    panic!("Expected '(' after 'if'");
                }
                self.next();
                let condition = self.expression();
                if self.current_token != Token::CloseParen {
                    panic!("Expected ')' after if condition");
                }
                self.next();
                let then_branch = Box::new(self.statement());
    
                let else_branch = if let Token::Else = self.current_token {
                    self.next();
                    Some(Box::new(self.statement()))
                } else {
                    None
                };
    
                Stmt::If {
                    condition,
                    then_branch,
                    else_branch,
                }
            }
            Token::While => {
                self.next(); // consume 'while'
                if self.current_token != Token::OpenParen {
                    panic!("Expected '(' after 'while'");
                }
                self.next(); // consume '('
                let condition = self.expression();
                if self.current_token != Token::CloseParen {
                    panic!("Expected ')' after while condition");
                }
                self.next(); // consume ')'
                let body = Box::new(self.statement());
                Stmt::While { condition, body }
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
                        op: BinOp::Add,
                        left: Box::new(node),
                        right: Box::new(self.primary()),
                    };
                }
                Token::Sub => {
                    self.next();
                    node = Expr::BinaryOp {
                        op: BinOp::Sub,
                        left: Box::new(node),
                        right: Box::new(self.primary()),
                    };
                }
                Token::Mul => {
                    self.next();
                    node = Expr::BinaryOp {
                        op: BinOp::Mul,
                        left: Box::new(node),
                        right: Box::new(self.primary()),
                    };
                }
                Token::Div => {
                    self.next();
                    node = Expr::BinaryOp {
                        op: BinOp::Div,
                        left: Box::new(node),
                        right: Box::new(self.primary()),
                    };
                }
                _ => {
                    break;
                }
            }
        }

        node
    }

    fn primary(&mut self) -> Expr {
        match &self.current_token {
            Token::Num(n) => {
                let value = *n;
                self.next();
                Expr::Number(value)
            }
            Token::OpenParen => {
                self.next(); // consume '('
                let expr = self.expression();
                if self.current_token == Token::CloseParen {
                    self.next(); // consume ')'
                    expr
                } else {
                    panic!("Expected closing parenthesis");
                }
            }
            _ => {
                panic!("Unexpected token in expression: {:?}", self.current_token);
            }
        }
    }

    fn next(&mut self) {
        self.current_token = self.lexer.next_token();
    }
}
