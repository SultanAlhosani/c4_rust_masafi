/// Represents the types of tokens the lexer can generate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Num(i32), // Integer number
    Identifier(String), // Variable or function name
    Return, // 'return' keyword
    If, // 'if' keyword
    Else, // 'else' keyword
    While, // 'while' keyword
    Let, // 'let' keyword
    OpenParen, // '(' character
    CloseParen, // ')' character
    OpenBrace, // '{' character
    CloseBrace, // '}' character
    OpenBracket, // '[' character
    CloseBracket, // ']' character
    Semicolon, // ';' character
    Assign, // '=' character
    Add, // '+' operator
    Sub, // '-' operator
    Mul, // '*' operator
    Div, // '/' operator
    Equal, // '==' operator
    NotEqual, // '!=' operator
    LessThan, // '<' operator
    GreaterThan, // '>' operator
    LessEqual, // '<=' operator
    GreaterEqual, // '>=' operator
    Eof, // End of file
    Unknown(char), // Unknown character (error)
    True, // 'true' keyword
    False, // 'false' keyword
    Char(char), // Single character
    Fn, // 'fn' keyword
    Comma, // ',' character
    And, // '&&' logical AND
    Or, // '||' logical OR
    Not, // '!' logical NOT
    Print, // 'print' keyword
    Enum, // 'enum' keyword
    StringLiteral(String), // String literal
    Sizeof, // 'sizeof' keyword
    Colon, // ':' character
    AddressOf, // '&' address-of operator
    Deref, // '*' dereference operator
    PlusPlus, // '++' increment operator
    MinusMinus, // '--' decrement operator
    QuestionMark, // '?' ternary operator
    Mod, // '%' modulus operator
    BitAnd, // '&' bitwise AND
    BitOr, // '|' bitwise OR
    BitXor, // '^' bitwise XOR
    BitNot, // '~' bitwise NOT
    Shl, // '<<' bitwise shift left
    Shr, // '>>' bitwise shift right
}

/// Lexer that tokenizes the input code.
pub struct Lexer {
    input: Vec<char>, // The input source code as a list of characters
    pos: usize, // Current position in the input
    line: usize, // Current line number
    col: usize, // Current column number
}

impl Lexer {
    /// Creates a new Lexer instance.
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(), // Convert input string into a character vector
            pos: 0, // Start at the first character
            line: 1, // Start at line 1
            col: 1, // Start at column 1
        }
    }

    /// Returns the next token in the input.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments(); // Skip any whitespace or comments

        if let Some(ch) = self.current_char() {
            match ch {
                '"' => { // Handle string literals
                    self.advance(); // Move past the opening quote
                    let mut string = String::new();
                    while let Some(c) = self.current_char() {
                        if c == '"' {
                            break; // End of string
                        } else if c == '\\' { // Handle escape sequences
                            self.advance();
                            if let Some(escaped) = self.current_char() {
                                match escaped {
                                    'n' => string.push('\n'),
                                    't' => string.push('\t'),
                                    '"' => string.push('"'),
                                    '\\' => string.push('\\'),
                                    _ => panic!("Unknown escape sequence \\{} at line {}, col {}", escaped, self.line, self.col),
                                }
                            }
                        } else {
                            string.push(c); // Add normal characters to the string
                        }
                        self.advance();
                    }
                    if self.current_char() != Some('"') {
                        panic!("Unterminated string literal at line {}, col {}", self.line, self.col);
                    }
                    self.advance(); // Move past the closing quote
                    Token::StringLiteral(string) // Return a string literal token
                }

                '\'' => { // Handle character literals
                    self.advance(); // Move past the opening quote
                    let ch = self.current_char().unwrap_or_else(|| {
                        panic!("Unterminated character literal at line {}, col {}", self.line, self.col)
                    });
                    self.advance();
                    if self.current_char() != Some('\'') {
                        panic!("Expected closing quote at line {}, col {}", self.line, self.col);
                    }
                    self.advance(); // Move past the closing quote
                    Token::Char(ch) // Return a character token
                }

                '0'..='9' => self.number(), // Number literals

                'a'..='z' | 'A'..='Z' | '_' => self.identifier_or_keyword(), // Identifiers or keywords

                '+' => { // Handle addition or increment
                    self.advance();
                    if self.current_char() == Some('+') {
                        self.advance();
                        Token::PlusPlus // Return increment operator
                    } else {
                        Token::Add // Return addition operator
                    }
                }

                '-' => { // Handle subtraction or decrement
                    self.advance();
                    if self.current_char() == Some('-') {
                        self.advance();
                        Token::MinusMinus // Return decrement operator
                    } else {
                        Token::Sub // Return subtraction operator
                    }
                }

                '*' => {
                    self.advance();
                    Token::Mul // Return multiplication operator
                }

                '/' => { // Handle division and comments
                    self.advance();
                    if self.match_char('/') { // Single-line comment
                        self.advance();
                        while let Some(c) = self.current_char() {
                            if c == '\n' {
                                break; // End of comment
                            }
                            self.advance();
                        }
                        self.next_token() // Continue processing after the comment
                    } else {
                        Token::Div // Return division operator
                    }
                }

                '%' => {
                    self.advance();
                    Token::Mod // Return modulus operator
                }

                '=' => { // Handle assignment or equality check
                    self.advance();
                    if self.current_char() == Some('=') {
                        self.advance();
                        Token::Equal // Return equality operator
                    } else {
                        Token::Assign // Return assignment operator
                    }
                }

                '!' => { // Handle logical NOT or inequality check
                    self.advance();
                    if self.current_char() == Some('=') {
                        self.advance();
                        Token::NotEqual // Return inequality operator
                    } else {
                        Token::Not // Return logical NOT operator
                    }
                }

                '<' => { // Handle less than or bitwise shift
                    self.advance();
                    if self.current_char() == Some('<') {
                        self.advance();
                        Token::Shl // Return shift left operator
                    } else if self.current_char() == Some('=') {
                        self.advance();
                        Token::LessEqual // Return less than or equal operator
                    } else {
                        Token::LessThan // Return less than operator
                    }
                }

                '>' => { // Handle greater than or bitwise shift
                    self.advance();
                    if self.current_char() == Some('>') {
                        self.advance();
                        Token::Shr // Return shift right operator
                    } else if self.current_char() == Some('=') {
                        self.advance();
                        Token::GreaterEqual // Return greater than or equal operator
                    } else {
                        Token::GreaterThan // Return greater than operator
                    }
                }

                '&' => { // Handle logical AND or bitwise AND
                    self.advance();
                    if self.current_char() == Some('&') {
                        self.advance();
                        Token::And // Return logical AND operator
                    } else {
                        Token::BitAnd // Return bitwise AND operator
                    }
                }

                '|' => { // Handle logical OR or bitwise OR
                    self.advance();
                    if self.current_char() == Some('|') {
                        self.advance();
                        Token::Or // Return logical OR operator
                    } else {
                        Token::BitOr // Return bitwise OR operator
                    }
                }

                '^' => {
                    self.advance();
                    Token::BitXor // Return bitwise XOR operator
                }

                '~' => {
                    self.advance();
                    Token::BitNot // Return bitwise NOT operator
                }

                '(' => { self.advance(); Token::OpenParen } // Left parenthesis
                ')' => { self.advance(); Token::CloseParen } // Right parenthesis
                '{' => { self.advance(); Token::OpenBrace } // Left brace
                '}' => { self.advance(); Token::CloseBrace } // Right brace
                '[' => { self.advance(); Token::OpenBracket } // Left bracket
                ']' => { self.advance(); Token::CloseBracket } // Right bracket

                ';' => { self.advance(); Token::Semicolon } // Semicolon
                ',' => { self.advance(); Token::Comma } // Comma
                ':' => { self.advance(); Token::Colon } // Colon
                '?' => { self.advance(); Token::QuestionMark } // Question mark

                _ => { // Unknown character
                    self.advance();
                    Token::Unknown(ch)
                }
            }
        } else {
            Token::Eof // End of file token
        }
    }

    // Parses a number from the current input.
    fn number(&mut self) -> Token {
        let mut value = 0;
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                value = value * 10 + (ch as i32 - '0' as i32); // Construct the number
                self.advance();
            } else {
                break; // End of number
            }
        }
        Token::Num(value) // Return the number token
    }

    // Parses an identifier or keyword from the current input.
    fn identifier_or_keyword(&mut self) -> Token {
        let start = self.pos;
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let word: String = self.input[start..self.pos].iter().collect(); // Extract the word

        match word.as_str() {
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "let" => Token::Let,
            "true" => Token::True,
            "false" => Token::False,
            "fn" => Token::Fn,
            "print" => Token::Print,
            "enum" => Token::Enum,
            "sizeof" => Token::Sizeof,
            "void" => Token::Identifier("void".to_string()),
            _ => Token::Identifier(word), // Return identifier token for variable names
        }
    }

    // Skips whitespace characters like spaces and newlines.
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace(); // Skip whitespace
            if self.current_char() == Some('/') && self.match_char('/') { // Check for comments
                self.advance();
                self.advance(); // Move past '//'
                while let Some(c) = self.current_char() {
                    if c == '\n' {
                        break; // End of comment
                    }
                    self.advance();
                }
            } else {
                break; // End of whitespace and comments
            }
        }
    }

    // Skips whitespace characters like spaces and newlines.
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch == '\n' {
                self.line += 1; // Increment line number on newline
                self.col = 1; // Reset column to 1
                self.advance();
            } else if ch.is_whitespace() {
                self.col += 1; // Increment column number
                self.advance();
            } else {
                break;
            }
        }
    }

    // Checks if the next character matches the expected character.
    fn match_char(&self, expected: char) -> bool {
        self.input.get(self.pos + 1) == Some(&expected)
    }

    // Advances to the next character in the input.
    fn advance(&mut self) {
        if let Some(ch) = self.input.get(self.pos) {
            if *ch == '\n' {
                self.line += 1; // Increment line number on newline
                self.col = 1; // Reset column to 1
            } else {
                self.col += 1; // Increment column number
            }
        }
        self.pos += 1; // Move to next character
    }

    // Returns the current character in the input.
    fn current_char(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    // Returns the current line and column position in the input.
    pub fn get_position(&self) -> (usize, usize) {
        (self.line, self.col)
    }
}
