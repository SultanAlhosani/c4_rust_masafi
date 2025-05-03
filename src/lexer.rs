#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Num(i32),
    Identifier(String),
    Return,
    If,
    Else,
    While,
    Let,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    Eof,
    Unknown(char),
    True,
    False,
    Char(char),
    Fn,
    Comma,
    And,     // &&
    Or,      // ||
    Not,     // !
    Print,   // ✅ NEW
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if let Some(ch) = self.current_char() {
            match ch {
                '\'' => {
                    self.advance();
                    let ch = self.current_char().unwrap_or_else(|| {
                        panic!("Unterminated character literal at line {}, col {}", self.line, self.col)
                    });
                    self.advance();
                    if self.current_char() != Some('\'') {
                        panic!("Expected closing quote at line {}, col {}", self.line, self.col);
                    }
                    self.advance();
                    Token::Char(ch)
                }
                '0'..='9' => self.number(),
                'a'..='z' | 'A'..='Z' | '_' => self.identifier_or_keyword(),
                '+' => { self.advance(); Token::Add }
                '-' => { self.advance(); Token::Sub }
                '*' => { self.advance(); Token::Mul }
                '/' => { self.advance(); Token::Div }
                '(' => { self.advance(); Token::OpenParen }
                ')' => { self.advance(); Token::CloseParen }
                '{' => { self.advance(); Token::OpenBrace }
                '}' => { self.advance(); Token::CloseBrace }
                ';' => { self.advance(); Token::Semicolon }
                '=' => {
                    self.advance();
                    if self.match_char('=') {
                        self.advance();
                        Token::Equal
                    } else {
                        Token::Assign
                    }
                }
                '!' => {
                    self.advance();
                    if self.match_char('=') {
                        self.advance();
                        Token::NotEqual
                    } else {
                        Token::Not
                    }
                }
                '<' => { self.advance(); Token::LessThan }
                '>' => { self.advance(); Token::GreaterThan }
                ',' => { self.advance(); Token::Comma }
                '&' => {
                    self.advance();
                    if self.match_char('&') {
                        self.advance();
                        Token::And
                    } else {
                        panic!("Unexpected character '&' at line {}, col {}", self.line, self.col);
                    }
                }
                '|' => {
                    self.advance();
                    if self.match_char('|') {
                        self.advance();
                        Token::Or
                    } else {
                        panic!("Unexpected character '|' at line {}, col {}", self.line, self.col);
                    }
                }
                _ => {
                    self.advance();
                    Token::Unknown(ch)
                }
            }
        } else {
            Token::Eof
        }
    }

    fn number(&mut self) -> Token {
        let mut value = 0;
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                value = value * 10 + (ch as i32 - '0' as i32);
                self.advance();
            } else {
                break;
            }
        }
        Token::Num(value)
    }

    fn identifier_or_keyword(&mut self) -> Token {
        let start = self.pos;
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let word: String = self.input[start..self.pos].iter().collect();

        match word.as_str() {
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "let" => Token::Let,
            "true" => Token::True,
            "false" => Token::False,
            "fn" => Token::Fn,
            "print" => Token::Print, // ✅ added
            _ => Token::Identifier(word),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
                self.advance();
            } else if ch.is_whitespace() {
                self.col += 1;
                self.advance();
            } else {
                break;
            }
        }
    }

    fn match_char(&self, expected: char) -> bool {
        self.input.get(self.pos) == Some(&expected)
    }

    fn advance(&mut self) {
        if let Some(ch) = self.input.get(self.pos) {
            if *ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        self.pos += 1;
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    pub fn get_position(&self) -> (usize, usize) {
        (self.line, self.col)
    }

    fn peek(&self) -> char {
        *self.input.get(self.pos + 1).unwrap_or(&'\0')
    }
}
