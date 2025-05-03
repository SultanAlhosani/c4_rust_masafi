use crate::ast::{Expr, Stmt, BinOp, UnOp};
use crate::lexer::{Lexer, Token};
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
        match &self.current_token {
            Token::Return => {
                self.next();
                let expr = self.expression();
                if self.current_token == Token::Semicolon {
                    self.next();
                }
                Stmt::Return(expr)
            }
            Token::Let => {
                self.next();
                let name = match &self.current_token {
                    Token::Identifier(n) => {
                        let n = n.clone();
                        self.next();
                        n
                    }
                    _ => panic!("Expected identifier after 'let' at line {}, column {}", line, col),
                };
                if self.current_token != Token::Assign {
                    panic!("Expected '=' after identifier at line {}, column {}", line, col);
                }
                self.next();
                let value = self.expression();
                if self.current_token == Token::Semicolon {
                    self.next();
                }
                Stmt::Let { name, value }
            }
            Token::Print => {
                self.next();
                if self.current_token != Token::OpenParen {
                    panic!("Expected '(' after 'print' at line {}, column {}", line, col);
                }
                self.next();
                let expr = self.expression();
                if self.current_token != Token::CloseParen {
                    panic!("Expected ')' after expression in 'print' at line {}, column {}", line, col);
                }
                self.next();
                if self.current_token == Token::Semicolon {
                    self.next();
                }
                Stmt::Print(expr)
            }
            Token::Identifier(name) => {
                let var_name = name.clone();
                self.next();
                if self.current_token != Token::Assign {
                    panic!("Expected '=' after identifier at line {}, column {}", line, col);
                }
                self.next();
                let value = self.expression();
                if self.current_token == Token::Semicolon {
                    self.next();
                }
                Stmt::Assign { name: var_name, value }
            }
            Token::If => {
                self.next();
                if self.current_token != Token::OpenParen {
                    panic!("Expected '(' after 'if' at line {}, column {}", line, col);
                }
                self.next();
                let condition = self.expression();
                if self.current_token != Token::CloseParen {
                    panic!("Expected ')' after condition at line {}, column {}", line, col);
                }
                self.next();
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
                if self.current_token != Token::OpenParen {
                    panic!("Expected '(' after 'while' at line {}, column {}", line, col);
                }
                self.next();
                let condition = self.expression();
                if self.current_token != Token::CloseParen {
                    panic!("Expected ')' after condition at line {}, column {}", line, col);
                }
                self.next();
                let body = Box::new(self.statement());
                Stmt::While { condition, body }
            }
            Token::OpenBrace => self.block(),
            Token::Fn => {
                self.next();
                let name = match &self.current_token {
                    Token::Identifier(n) => {
                        let n = n.clone();
                        self.next();
                        n
                    }
                    _ => panic!("Expected function name after 'fn' at line {}, column {}", line, col),
                };
                if self.current_token != Token::OpenParen {
                    panic!("Expected '(' after function name at line {}, column {}", line, col);
                }
                self.next();
                let mut params = Vec::new();
                while self.current_token != Token::CloseParen {
                    if let Token::Identifier(param) = &self.current_token {
                        params.push(param.clone());
                        self.next();
                        if self.current_token == Token::Comma {
                            self.next();
                        } else if self.current_token != Token::CloseParen {
                            panic!("Expected ',' or ')' in parameter list at line {}, column {}", line, col);
                        }
                    } else {
                        panic!("Expected parameter name at line {}, column {}", line, col);
                    }
                }
                self.next();
                let body = Box::new(self.block());
                Stmt::Function { name, params, body }
            }
            _ => panic!("Unexpected token: {:?} at line {}, column {}", self.current_token, line, col),
        }
    }

    fn expression(&mut self) -> Expr {
        self.parse_logic_or()
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
        while matches!(
            self.current_token,
            Token::Equal
                | Token::NotEqual
                | Token::LessThan
                | Token::GreaterThan
                | Token::LessEqual
                | Token::GreaterEqual
        ) {
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
            lhs = Expr::BinaryOp {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
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
        let mut lhs = self.parse_primary();
        while matches!(self.current_token, Token::Mul | Token::Div) {
            let op = match self.current_token {
                Token::Mul => BinOp::Mul,
                Token::Div => BinOp::Div,
                _ => unreachable!(),
            };
            self.next();
            let rhs = self.parse_primary();
            lhs = Expr::BinaryOp { op, left: Box::new(lhs), right: Box::new(rhs) };
        }
        lhs
    }

    fn parse_primary(&mut self) -> Expr {
        let (line, col) = self.lexer.get_position();
        if self.current_token == Token::Not {
            self.next();
            let inner = self.parse_primary();
            return Expr::UnaryOp { op: UnOp::Not, expr: Box::new(inner) };
        }
        match &self.current_token {
            Token::Num(n) => { let val = *n; self.next(); Expr::Number(val) }
            Token::True => { self.next(); Expr::Boolean(true) }
            Token::False => { self.next(); Expr::Boolean(false) }
            Token::Char(c) => { let ch = *c; self.next(); Expr::Char(ch) }
            Token::Identifier(name) => {
                let var_name = name.clone();
                self.next();
                if self.current_token == Token::OpenParen {
                    self.next();
                    let mut args = Vec::new();
                    while self.current_token != Token::CloseParen {
                        args.push(self.expression());
                        if self.current_token == Token::Comma {
                            self.next();
                        } else if self.current_token != Token::CloseParen {
                            panic!("Expected ',' or ')' in function call at line {}, column {}", line, col);
                        }
                    }
                    self.next();
                    Expr::FunctionCall { name: var_name, args }
                } else {
                    Expr::Variable(var_name)
                }
            }
            Token::OpenParen => {
                self.next();
                let expr = self.expression();
                if self.current_token != Token::CloseParen {
                    panic!("Expected ')' after expression at line {}, column {}", line, col);
                }
                self.next();
                expr
            }
            _ => panic!("Unexpected token in expression at line {}, column {}: {:?}", line, col, self.current_token),
        }
    }

    fn block(&mut self) -> Stmt {
        if self.current_token != Token::OpenBrace {
            return self.statement();
        }
        self.next();
        let mut statements = Vec::new();
        while self.current_token != Token::CloseBrace {
            statements.push(self.statement());
        }
        self.next();
        Stmt::Block(statements)
    }
}
