#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Num(i32),
    Id(String),
    Return,
    If,
    Else,
    While,
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
    Less,
    Greater,
    Equal,
    NotEqual,
    Eof,
    Unknown(char),
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if let Some(ch) = self.current_char() {
            match ch {
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
                        Token::Unknown('!')
                    }
                }
                '<' => { self.advance(); Token::Less }
                '>' => { self.advance(); Token::Greater }
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
            _ => Token::Id(word),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
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
        self.pos += 1;
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }
}
