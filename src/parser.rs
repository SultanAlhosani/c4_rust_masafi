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
                self.next(); // consume 'return'
                let expr = self.expression(); // parse the expression
                if self.current_token == Token::Semicolon {
                    self.next(); // consume ';'
                } else {
                    panic!("Expected ';' after return value");
                }
                Stmt::Return(expr)
            }
            _ => {
                panic!("Unexpected token in statement: {:?}", self.current_token);
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
