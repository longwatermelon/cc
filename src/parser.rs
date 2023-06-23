use crate::error::Error;
use crate::lexer::*;
use crate::node::*;

pub struct Parser {
    lexer: Lexer,
    curr: Token,
    prev: Token
}

impl Parser {
    pub fn new(contents: String) -> Result<Self, Error> {
        let mut lexer: Lexer = Lexer::new(contents);
        let curr = lexer.next()?;
        Ok(Self {
            lexer,
            curr: curr.clone(),
            prev: curr
        })
    }

    pub fn parse(&mut self) -> Result<Node, Error> {
        let mut cpd_values: Vec<Node> = Vec::new();

        loop {
            cpd_values.push(
                match self.parse_expr()? {
                    Some(x) => x,
                    None => break
                }
            );

            self.expect(TokenType::Semi)?;
        }

        if cpd_values.is_empty() {
            cpd_values.push(Node::new(NodeVariant::Noop, 0));
        }

        Ok(Node::new(NodeVariant::Cpd { values: cpd_values }, 0))
    }

    fn expect(&mut self, ttype: TokenType) -> Result<(), Error> {
        if self.curr.ttype == ttype {
            self.prev = self.curr.clone();
            self.curr = self.lexer.next()?;
            Ok(())
        } else {
            Err(Error::new(format!(
                "expected token type {:?}, got {:?}.",
                ttype, self.curr.ttype
            ), self.curr.line))
        }
    }

    fn parse_expr(&mut self) -> Result<Option<Node>, Error> {
        Ok(
            match self.curr.ttype {
                TokenType::Str => Some(self.parse_str()?),
                TokenType::Int => Some(self.parse_int()?),
                TokenType::Id => Some(self.parse_id()?),
                TokenType::Lbrace => {
                    self.expect(TokenType::Lbrace)?;
                    let node = self.parse()?;
                    self.expect(TokenType::Rbrace)?;
                    Some(node)
                },
                _ => None
            }
        )
    }

    fn parse_int(&mut self) -> Result<Node, Error> {
        let int_value: i32 = match self.curr.value.parse::<i32>() {
            Ok(x) => x,
            Err(e) => return Err(Error::new(e.to_string(), self.curr.line))
        };
        self.expect(TokenType::Int)?;
        Ok(Node::new(NodeVariant::Int { value: int_value }, self.curr.line))
    }

    fn parse_str(&mut self) -> Result<Node, Error> {
        self.expect(TokenType::Str)?;
        Ok(Node::new(NodeVariant::Str { value: self.prev.value.clone() }, self.curr.line))
    }

    fn parse_id(&mut self) -> Result<Node, Error> {
        if self.curr.value == "if" {
            return self.parse_if();
        }

        self.expect(TokenType::Id)?;

        if self.curr.ttype == TokenType::Lparen {
            self.parse_fcall()
        } else {
            self.parse_var()
        }
    }

    fn parse_fcall(&mut self) -> Result<Node, Error> {
        let name: String = self.prev.value.clone();
        let mut args: Vec<Node> = Vec::new();
        let line: usize = self.curr.line;

        self.expect(TokenType::Lparen)?;
        loop {
            match self.parse_expr()? {
                Some(expr) => args.push(expr),
                None => break
            };

            if self.curr.ttype != TokenType::Rparen {
                self.expect(TokenType::Comma)?;
            }
        }
        self.expect(TokenType::Rparen)?;

        Ok(Node::new(NodeVariant::Fcall { name, args }, line))
    }

    fn parse_var(&mut self) -> Result<Node, Error> {
        let name: String = self.prev.value.clone();
        let line: usize = self.curr.line;

        match self.curr.ttype {
            TokenType::Id => {
                self.parse_vardef()
            },
            TokenType::Equal => self.parse_assign(),
            _ => {
                Ok(Node::new(NodeVariant::Var { name }, line))
            }
        }
    }

    fn parse_vardef(&mut self) -> Result<Node, Error> {
        let name: String = self.curr.value.clone();
        let dtype: Dtype = Dtype::from_str(self.prev.value.clone());
        let line: usize = self.curr.line;

        self.expect(TokenType::Id)?;
        self.expect(TokenType::Equal)?;
        Ok(
            Node::new(NodeVariant::Vardef {
                name: name.clone(),
                value:
                    match self.parse_expr()? {
                        Some(x) => x,
                        None => return Err(Error::new(format!("no expression in definition of '{}'.", name), line))
                    },
                dtype
                }, line
            )
        )
    }

    fn parse_assign(&mut self) -> Result<Node, Error> {
        let line: usize = self.curr.line;
        let l: Node = Node::new(NodeVariant::Var { name: self.prev.value.clone() }, line);
        self.expect(TokenType::Equal)?;
        let r: Node = Node::new(self.parse_expr()?.unwrap().variant.as_ref().clone(), line);

        Ok(Node::new(NodeVariant::Assign { l, r }, line))
    }

    fn parse_if(&mut self) -> Result<Node, Error> {
        let line: usize = self.curr.line;
        self.expect(TokenType::Id)?;

        self.expect(TokenType::Lparen)?;
        let cond: Node = self.parse_expr()?.unwrap();
        self.expect(TokenType::Rparen)?;

        Ok(
            Node::new(
                NodeVariant::If { cond, body: self.parse_expr()?.unwrap() },
                line
            )
        )
    }
}

