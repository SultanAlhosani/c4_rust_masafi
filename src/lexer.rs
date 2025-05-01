/// Lexer analyzer for tokenizing the input source code.
/// It converts the source code into a stream of tokens that can be used by the parser.

#[derive(Debug, Clone, PartialEq, Eq)]
/// All tokens that the lexer can recognize.
/// This includes keywords, identifiers, literals, and operators.
pub enum Token {
    Num(i32),
    Identifier(String),
    Return,
    If,
    Else,
    While,
    Let, // <-- Added Let keyword
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Assign, // (=)
    Add,
    Sub,
    Mul,
    Div,
    Equal,       // (==)
    NotEqual,    // (!=)
    LessThan,    // (<)
    GreaterThan, // (>)
    Eof,
    Unknown(char),
    True,
    False,
    Char(char),
    Fn,
    Comma,
}
/// A lexical analyzer (lexer) for tokenizing source code.
///
/// The lexer processes the input source code and produces a sequence of tokens
/// that represent the smallest units of meaning in the code.
pub struct Lexer {
    /// The input source code as a vector of characters.
    input: Vec<char>,

    /// The current position in the input.
    pos: usize,
}

impl Lexer {
    /// Creates a new `Lexer` instance.
    ///
    /// # Parameters
    /// - `input`: The source code to tokenize.
    ///
    /// # Returns
    /// A new `Lexer` instance.
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    /// Retrieves the next token from the input.
    ///
    /// This function skips over whitespace and identifies the next meaningful
    /// token in the source code.
    ///
    /// # Returns
    /// The next `Token` in the input.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if let Some(ch) = self.current_char() {
            match ch {
                '\'' => {
                    self.advance(); // Consume opening '

                    let ch = match self.current_char() {
                        Some(c) => c,
                        None => panic!("Unterminated character literal"),
                    };

                    self.advance(); // Consume the character

                    if self.current_char() != Some('\'') {
                        panic!("Expected closing single quote after character");
                    }

                    self.advance(); // Consume closing '

                    Token::Char(ch)
                }
                '0'..='9' => self.number(),
                'a'..='z' | 'A'..='Z' | '_' => self.identifier_or_keyword(),
                '+' => {
                    self.advance();
                    Token::Add
                }
                '-' => {
                    self.advance();
                    Token::Sub
                }
                '*' => {
                    self.advance();
                    Token::Mul
                }
                '/' => {
                    self.advance();
                    Token::Div
                }
                '(' => {
                    self.advance();
                    Token::OpenParen
                }
                ')' => {
                    self.advance();
                    Token::CloseParen
                }
                '{' => {
                    self.advance();
                    Token::OpenBrace
                }
                '}' => {
                    self.advance();
                    Token::CloseBrace
                }
                ';' => {
                    self.advance();
                    Token::Semicolon
                }
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
                '<' => {
                    self.advance();
                    Token::LessThan
                }
                '>' => {
                    self.advance();
                    Token::GreaterThan
                }
                ',' => {
                    self.advance();
                    Token::Comma
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

    /// Parses a numeric literal from the input.
    ///
    /// This function reads consecutive digits and converts them into an integer.
    ///
    /// # Returns
    /// A `Token::Num` containing the parsed number.
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

    /// Parses an identifier or keyword from the input.
    ///
    /// This function reads consecutive alphanumeric characters or underscores
    /// and determines whether the result is a keyword or an identifier.
    ///
    /// # Returns
    /// A `Token` representing the identifier or keyword.
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
            _ => Token::Identifier(word),
        }
    }

    /// Skips over whitespace characters in the input.
    ///
    /// This function advances the position in the input until a non-whitespace
    /// character is encountered.
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Checks if the current character matches the expected character.
    ///
    /// # Parameters
    /// - `expected`: The character to match.
    ///
    /// # Returns
    /// `true` if the current character matches, `false` otherwise.
    fn match_char(&self, expected: char) -> bool {
        self.input.get(self.pos) == Some(&expected)
    }

    /// Advances the current position in the input.
    ///
    /// This function moves the position forward by one character.
    fn advance(&mut self) {
        self.pos += 1;
    }

    /// Retrieves the current character from the input.
    ///
    /// # Returns
    /// The current character, or `None` if at the end of the input.
    fn current_char(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }
}
