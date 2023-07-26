use crate::error::{Error, ErrorType};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    Id,
    Str,
    Int,
    Char,
    Semi,
    Lparen,
    Rparen,
    Lbrace,
    Rbrace,
    Equal,
    Comma,
    Star,
    Amp,
    Plus,
    Minus,
    Div,
    EqualCmp,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Dot,
    Arrow,
    And,
    Or,
    Eof
}

#[derive(Clone)]
pub struct Token {
    pub ttype: TokenType,
    pub value: String,
    pub line: usize
}

#[derive(Clone)]
pub struct Lexer {
    contents: String,
    pub line: usize,
    index: usize,
    ch: char
}

impl TokenType {
    pub fn is_binop(&self) -> bool {
        matches!(self,
            TokenType::Plus  |
            TokenType::Minus |
            TokenType::Star  |
            TokenType::Div   |
            TokenType::Less  |
            TokenType::Greater |
            TokenType::LessEqual |
            TokenType::GreaterEqual |
            TokenType::EqualCmp |
            TokenType::Dot |
            TokenType::Equal |
            TokenType::And |
            TokenType::Or |
            TokenType::Arrow
        )
    }

    /// High weight binops will be the operands of low weight binops.
    pub fn binop_weight(&self) -> i32 {
        match self {
            TokenType::Dot |
            TokenType::Arrow => 3,
            TokenType::Plus  |
            TokenType::Minus |
            TokenType::Star  |
            TokenType::Div  => 2,
            TokenType::Less  |
            TokenType::Greater |
            TokenType::LessEqual |
            TokenType::GreaterEqual |
            TokenType::EqualCmp |
            TokenType::Equal => 1,
            TokenType::And |
            TokenType::Or => 0,
            _ => panic!()
        }
    }

    pub fn is_unop(&self) -> bool {
        matches!(self,
            TokenType::Star |
            TokenType::Amp
        )
    }
}

impl Token {
    pub fn new(ttype: TokenType, value: String, line: usize) -> Self {
        Self {
            ttype, value, line
        }
    }
}

impl Lexer {
    pub fn new(contents: &str) -> Self {
        Self {
            contents: contents.to_string(),
            line: 1,
            index: 0,
            ch: contents.chars().next().unwrap()
        }
    }

    pub fn next(&mut self) -> Result<Token, Error> {
        while self.index < self.contents.len() - 1 {
            while self.ch.is_whitespace() && self.ch != '\n' {
                self.advance();
            }

            if self.ch.is_numeric() {
                return Ok(Token::new(TokenType::Int, self.collect_num(), self.line));
            }

            if self.ch.is_alphabetic() {
                return Ok(Token::new(TokenType::Id, self.collect_id(), self.line));
            }

            if self.ch == '"' {
                return Ok(Token::new(TokenType::Str, self.collect_str(), self.line));
            }

            if self.ch == '\'' {
                self.advance();
                let ch: char = self.ch;
                self.advance();
                self.advance();
                return Ok(Token::new(TokenType::Char, ch.to_string(), self.line));
            }

            match self.ch {
                ';' => return Ok(self.advance_with_tok(TokenType::Semi)),
                '(' => return Ok(self.advance_with_tok(TokenType::Lparen)),
                ')' => return Ok(self.advance_with_tok(TokenType::Rparen)),
                '{' => return Ok(self.advance_with_tok(TokenType::Lbrace)),
                '}' => return Ok(self.advance_with_tok(TokenType::Rbrace)),
                '=' => {
                    self.advance();
                    if self.ch == '=' {
                        return Ok(self.advance_with_tok(TokenType::EqualCmp));
                    } else {
                        return Ok(Token::new(TokenType::Equal, "=".to_string(), self.line));
                    }
                },
                ',' => return Ok(self.advance_with_tok(TokenType::Comma)),
                '*' => return Ok(self.advance_with_tok(TokenType::Star)),
                '&' => {
                    self.advance();
                    if self.ch == '&' {
                        return Ok(self.advance_with_tok(TokenType::And))
                    } else {
                        return Ok(Token::new(TokenType::Amp, "&".to_string(), self.line))
                    }
                },
                '|' => {
                    self.advance();
                    if self.ch == '|' {
                        return Ok(self.advance_with_tok(TokenType::Or))
                    } else {
                        return Err(Error::new(
                            ErrorType::UnrecognizedToken(self.ch),
                            self.line
                        ))
                    }
                },
                '+' => return Ok(self.advance_with_tok(TokenType::Plus)),
                '-' => {
                    self.advance();
                    if self.ch == '>' {
                        return Ok(self.advance_with_tok(TokenType::Arrow))
                    } else {
                        return Ok(Token::new(TokenType::Minus, "-".to_string(), self.line))
                    }
                },
                '/' => return Ok(self.advance_with_tok(TokenType::Div)),
                '<' => {
                    self.advance();
                    if self.ch == '=' {
                        return Ok(self.advance_with_tok(TokenType::LessEqual));
                    } else {
                        return Ok(Token::new(TokenType::Less, "<".to_string(), self.line));
                    }
                },
                '>' => {
                    self.advance();
                    if self.ch == '=' {
                        return Ok(self.advance_with_tok(TokenType::GreaterEqual));
                    } else {
                        return Ok(Token::new(TokenType::Greater, ">".to_string(), self.line));
                    }
                },
                '.' => return Ok(self.advance_with_tok(TokenType::Dot)),
                '\n' => {
                    self.line += 1;
                    self.advance()
                },
                _ => return Err(Error::new(
                        ErrorType::UnrecognizedToken(self.ch),
                        self.line
                    ))
            }
        }

        Ok(Token::new(TokenType::Eof, String::new(), self.line))
    }

    pub fn peek(&mut self, count: usize) -> Result<Token, Error> {
        let mut copy: Lexer = self.clone();
        for _ in 0..count - 1 {
            copy.next()?;
        }

        copy.next()
    }

    fn advance(&mut self) {
        if self.index < self.contents.len() {
            self.index += 1;
            self.ch = self.contents.chars().nth(self.index).unwrap();
        }
    }

    fn collect_num(&mut self) -> String {
        let mut res: String = String::new();

        while self.ch.is_numeric() || self.ch == '.' {
            res.push(self.ch);
            self.advance();
        }

        res
    }

    fn collect_str(&mut self) -> String {
        let mut res: String = String::new();
        self.advance();

        while self.ch != '"' {
            res.push(self.ch);
            self.advance();
        }

        self.advance();
        res
    }

    fn collect_id(&mut self) -> String {
        let mut res: String = String::new();

        while self.ch.is_alphanumeric() || self.ch == '_' {
            res.push(self.ch);
            self.advance();
        }

        res
    }

    fn advance_with_tok(&mut self, ttype: TokenType) -> Token {
        let ch: char = self.ch;
        self.advance();
        Token::new(ttype, String::from(ch), self.line)
    }
}

