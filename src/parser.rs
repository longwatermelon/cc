use crate::error::Error;
use crate::lexer::*;
use crate::node::*;

pub struct Parser {
    lexer: Lexer,
    curr: Token,
    prev: Token,
    prev_expr: Node
}

impl Parser {
    pub fn new(contents: String) -> Result<Self, Error> {
        let mut lexer: Lexer = Lexer::new(contents);
        let curr = lexer.next()?;
        Ok(Self {
            lexer,
            curr: curr.clone(),
            prev: curr,
            prev_expr: Node::new(NodeVariant::Noop, 0)
        })
    }

    pub fn parse(&mut self) -> Result<Node, Error> {
        let line: usize = self.curr.line;
        let mut cpd_values: Vec<Node> = Vec::new();

        loop {
            cpd_values.push(
                match self.parse_expr(false)? {
                    Some(x) => x,
                    None => break
                }
            );

            if self.prev.ttype != TokenType::Rbrace && self.prev.ttype != TokenType::Semi {
                self.expect(TokenType::Semi)?;
            }
        }

        if cpd_values.is_empty() {
            cpd_values.push(Node::new(NodeVariant::Noop, line));
        }

        Ok(Node::new(NodeVariant::Cpd { values: cpd_values }, line))
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

    fn parse_expr(&mut self, only_one: bool) -> Result<Option<Node>, Error> {
        while self.curr.ttype == TokenType::Semi {
            self.expect(TokenType::Semi)?;
        }

        let mut n: Option<Node> = match self.curr.ttype {
            TokenType::Str => Some(self.parse_str()?),
            TokenType::Int => Some(self.parse_int()?),
            TokenType::Id => Some(self.parse_id()?),
            TokenType::Lbrace => {
                self.expect(TokenType::Lbrace)?;
                let node = self.parse()?;
                self.expect(TokenType::Rbrace)?;

                Some(node)
            },
            TokenType::Lparen => {
                self.expect(TokenType::Lparen)?;
                let expr = self.parse_expr(false)?.unwrap();
                self.expect(TokenType::Rparen)?;

                Some(expr)
            },
            _ => None
        };

        if self.curr.is_unop() {
            n = Some(self.parse_unop()?);
        }

        if let Some(n) = &n {
            self.prev_expr = n.clone();
        }

        if !only_one {
            if self.curr.is_binop() {
                return Ok(Some(self.parse_binop()?));
            }
        }

        Ok(n)
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
            self.parse_if()
        } else if self.curr.value == "return" {
            self.parse_return()
        } else if self.curr.value == "struct" {
            self.parse_struct()
        } else {
            match self.lexer.peek(1)?.ttype {
                TokenType::Lparen => self.parse_fcall(),
                                _ => self.parse_var()
            }
        }
    }

    fn parse_fcall(&mut self) -> Result<Node, Error> {
        let name: String = self.curr.value.clone();
        self.expect(TokenType::Id)?;

        let mut args: Vec<Node> = Vec::new();
        let line: usize = self.curr.line;

        self.expect(TokenType::Lparen)?;
        loop {
            match self.parse_expr(false)? {
                Some(expr) => args.push(expr),
                None => break
            };

            if self.curr.ttype != TokenType::Rparen {
                self.expect(TokenType::Comma)?;
            } else {
                break;
            }
        }
        self.expect(TokenType::Rparen)?;

        Ok(Node::new(NodeVariant::Fcall { name, args }, line))
    }

    fn parse_fdef(&mut self, rtype: Dtype) -> Result<Node, Error> {
        let line: usize = self.curr.line;
        let name: String = self.prev.value.clone();
        let mut params: Vec<Node> = Vec::new();

        self.expect(TokenType::Lparen)?;
        loop {
            match self.parse_expr(false)? {
                Some(expr) => params.push(expr),
                None => break
            };

            if self.curr.ttype != TokenType::Rparen {
                self.expect(TokenType::Comma)?;
            }
        }
        self.expect(TokenType::Rparen)?;

        self.expect(TokenType::Lbrace)?;
        let body: Node = self.parse()?;
        self.expect(TokenType::Rbrace)?;

        Ok(Node::new(NodeVariant::Fdef { name, params, body, rtype }, line))
    }

    fn parse_return(&mut self) -> Result<Node, Error> {
        self.expect(TokenType::Id)?;
        Ok(Node::new(NodeVariant::Return { value: self.parse_expr(false)?.unwrap() }, self.curr.line))
    }

    fn parse_var(&mut self) -> Result<Node, Error> {
        // Start on name
        let name: String = self.curr.value.clone();

        if Dtype::new(name.clone()).is_ok() {
            self.parse_vardef()
        } else {
            self.expect(TokenType::Id)?;
            Ok(Node::new(NodeVariant::Var { name }, self.curr.line))
        }
    }

    fn parse_unop(&mut self) -> Result<Node, Error> {
        self.expect(self.curr.ttype)?;
        Ok(Node::new(NodeVariant::Unop { utype: self.prev.ttype, r: self.parse_expr(true)?.unwrap() }, self.curr.line))
    }

    fn parse_vardef(&mut self) -> Result<Node, Error> {
        let mut dtype: Dtype = Dtype::new(self.curr.value.clone())?;
        if let Dtype::Struct { name } = &mut dtype {
            self.expect(self.curr.ttype)?;
            *name = self.curr.value.clone();
        }

        self.expect(TokenType::Id)?;

        let var: Node = if self.curr.is_unop() {
            self.parse_unop()?
        } else {
            self.parse_var()?
        };

        let line: usize = self.curr.line;

        match self.curr.ttype {
            TokenType::Equal => {
                self.expect(TokenType::Equal)?;
                Ok(
                    Node::new(NodeVariant::Vardef {
                        var: var.clone(),
                        value:
                            match self.parse_expr(false)? {
                                Some(x) => x,
                                None => return Err(Error::new(format!("no expression in definition of '{}'.", var.var_name()), line))
                            },
                        dtype
                        }, line
                    )
                )
            },
            TokenType::Lparen => {
                self.parse_fdef(dtype)
            },
            _ => Ok(Node::new(NodeVariant::Vardef { var, value: Node::new(NodeVariant::Noop, 0), dtype }, line))
        }
    }

    fn parse_if(&mut self) -> Result<Node, Error> {
        let line: usize = self.curr.line;
        self.expect(TokenType::Id)?;

        self.expect(TokenType::Lparen)?;
        let cond: Node = self.parse_expr(false)?.unwrap();
        self.expect(TokenType::Rparen)?;

        Ok(
            Node::new(
                NodeVariant::If { cond, body: self.parse_expr(false)?.unwrap() },
                line
            )
        )
    }

    fn parse_binop(&mut self) -> Result<Node, Error> {
        let line: usize = self.curr.line;
        let btype: TokenType = self.curr.ttype;
        self.expect(btype)?;

        let l: Node = self.prev_expr.clone();
        let r: Node = self.parse_expr(true)?.unwrap();

        let n: Node = Node::new(NodeVariant::Binop { btype, l, r },line);

        if self.curr.is_binop() {
            self.prev_expr = n;
            self.parse_binop()
        } else {
            Ok(n)
        }
    }

    fn parse_struct(&mut self) -> Result<Node, Error> {
        if self.lexer.peek(2)?.ttype == TokenType::Lbrace {
            self.parse_struct_def()
        } else {
            self.parse_vardef()
        }
    }

    fn parse_struct_def(&mut self) -> Result<Node, Error> {
        let line: usize = self.curr.line;
        self.expect(TokenType::Id)?; // struct keyword

        let name: String = self.curr.value.clone();
        self.expect(TokenType::Id)?; // struct name

        self.expect(TokenType::Lbrace)?;
        let mut fields: Vec<Node> = Vec::new();
        loop {
            if let Some(expr) = self.parse_expr(false)? {
                fields.push(expr);
                self.expect(TokenType::Semi)?;
            } else {
                break;
            }
        }
        self.expect(TokenType::Rbrace)?;
        self.expect(TokenType::Semi)?;

        Ok(Node::new(NodeVariant::Struct { name, fields }, line))
    }
}

