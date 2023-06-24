use crate::error::Error;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    Id,
    Str,
    Int,
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

impl Token {
    pub fn new(ttype: TokenType, value: String, line: usize) -> Self {
        Self {
            ttype, value, line
        }
    }

    pub fn is_binop(&self) -> bool {
        match self.ttype {
            TokenType::Plus |
            TokenType::Minus |
            TokenType::Star |
            TokenType::Div => true,
            _ => false
        }
    }
}

impl Lexer {
    pub fn new(contents: String) -> Self {
        Self {
            contents: contents.clone(),
            line: 1,
            index: 0,
            ch: contents.chars().nth(0).unwrap()
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

            match self.ch {
                ';' => return Ok(self.advance_with_tok(TokenType::Semi)),
                '(' => return Ok(self.advance_with_tok(TokenType::Lparen)),
                ')' => return Ok(self.advance_with_tok(TokenType::Rparen)),
                '{' => return Ok(self.advance_with_tok(TokenType::Lbrace)),
                '}' => return Ok(self.advance_with_tok(TokenType::Rbrace)),
                '=' => return Ok(self.advance_with_tok(TokenType::Equal)),
                ',' => return Ok(self.advance_with_tok(TokenType::Comma)),
                '*' => return Ok(self.advance_with_tok(TokenType::Star)),
                '&' => return Ok(self.advance_with_tok(TokenType::Amp)),
                '+' => return Ok(self.advance_with_tok(TokenType::Plus)),
                '-' => return Ok(self.advance_with_tok(TokenType::Minus)),
                '/' => return Ok(self.advance_with_tok(TokenType::Div)),
                '\n' => {
                    self.line += 1;
                    self.advance()
                },
                _ => return Err(Error::new(format!("unrecognized token '{}'.", self.ch), self.line))
            }
        }

        return Ok(Token::new(TokenType::Eof, String::new(), self.line));
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

        return res;
    }

    fn collect_str(&mut self) -> String {
        let mut res: String = String::new();
        self.advance();

        while self.ch != '"' {
            res.push(self.ch);
            self.advance();
        }

        self.advance();
        return res;
    }

    fn collect_id(&mut self) -> String {
        let mut res: String = String::new();

        while self.ch.is_alphanumeric() || self.ch == '_' {
            res.push(self.ch);
            self.advance();
        }

        return res;
    }

    fn advance_with_tok(&mut self, ttype: TokenType) -> Token {
        let ch: char = self.ch;
        self.advance();
        return Token::new(ttype, String::from(ch), self.line);
    }
}

