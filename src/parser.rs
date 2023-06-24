use crate::error::Error;
use crate::lexer::*;
use crate::node::*;

pub struct Parser {
    lexer: Lexer,
    curr: Token,
    prev: Token,
    prev_expr: Node,
    ignore_ops: bool
}

impl Parser {
    pub fn new(contents: String) -> Result<Self, Error> {
        let mut lexer: Lexer = Lexer::new(contents);
        let curr = lexer.next()?;
        Ok(Self {
            lexer,
            curr: curr.clone(),
            prev: curr,
            prev_expr: Node::new(NodeVariant::Noop, 0),
            ignore_ops: false
        })
    }

    pub fn parse(&mut self) -> Result<Node, Error> {
        let line: usize = self.curr.line;
        let mut cpd_values: Vec<Node> = Vec::new();

        loop {
            cpd_values.push(
                match self.parse_expr()? {
                    Some(x) => x,
                    None => break
                }
            );

            println!("here");

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

    fn parse_expr(&mut self) -> Result<Option<Node>, Error> {
        while self.curr.ttype == TokenType::Semi {
            self.expect(TokenType::Semi)?;
        }

        let n: Node = match self.curr.ttype {
            TokenType::Str => self.parse_str()?,
            TokenType::Int => self.parse_int()?,
            TokenType::Id => self.parse_id()?,
            TokenType::Star => self.parse_deref()?,
            TokenType::Amp => self.parse_ref()?,
            TokenType::Lbrace => {
                let prev: bool = self.ignore_ops;
                self.ignore_ops = false;

                self.expect(TokenType::Lbrace)?;
                let node = self.parse()?;
                self.expect(TokenType::Rbrace)?;

                self.ignore_ops = prev;
                node
            },
            TokenType::Lparen => {
                let prev: bool = self.ignore_ops;
                self.ignore_ops = false;

                self.expect(TokenType::Lparen)?;
                let expr = self.parse_expr()?.unwrap();
                self.expect(TokenType::Rparen)?;

                self.ignore_ops = prev;
                expr
            },
            _ => return Ok(None)
        };

        self.prev_expr = n.clone();

        if !self.ignore_ops {
            if self.curr.is_binop() {
                return Ok(Some(self.parse_binop()?));
            }
        } else {
            self.ignore_ops = false;
        }

        Ok(Some(n))
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
            match self.parse_expr()? {
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
            match self.parse_expr()? {
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
        Ok(Node::new(NodeVariant::Return { value: self.parse_expr()?.unwrap() }, self.curr.line))
    }

    fn parse_var(&mut self) -> Result<Node, Error> {
        // Start on name
        let name: String = self.curr.value.clone();

        if Dtype::new(name.clone()).is_ok() {
            self.parse_vardef()
        } else {
            self.expect(TokenType::Id)?;

            if self.curr.ttype == TokenType::Equal && !self.ignore_ops {
                self.parse_assign()
            } else {
                Ok(Node::new(NodeVariant::Var { name }, self.curr.line))
            }
        }
    }

    fn parse_ref(&mut self) -> Result<Node, Error> {
        self.expect(TokenType::Amp)?;
        Ok(Node::new(NodeVariant::Ref { value: self.parse_expr()?.unwrap() }, self.curr.line))
    }

    fn parse_deref(&mut self) -> Result<Node, Error> {
        self.expect(TokenType::Star)?;
        Ok(Node::new(NodeVariant::Deref { value: self.parse_expr()?.unwrap() }, self.curr.line))
    }

    fn parse_vardef(&mut self) -> Result<Node, Error> {
        let mut dtype: Dtype = Dtype::new(self.curr.value.clone())?;
        if let Dtype::Struct { name } = &mut dtype {
            self.expect(self.curr.ttype)?;
            *name = self.curr.value.clone();
        }

        self.expect(TokenType::Id)?;

        let var: Node = match self.curr.ttype {
            TokenType::Id => self.parse_var()?,
            TokenType::Star => self.parse_deref()?,
            TokenType::Amp => self.parse_ref()?,
            _ => panic!()
        };
        let line: usize = self.curr.line;

        match self.curr.ttype {
            TokenType::Equal => {
                self.expect(TokenType::Equal)?;
                Ok(
                    Node::new(NodeVariant::Vardef {
                        var: var.clone(),
                        value:
                            match self.parse_expr()? {
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

    fn parse_binop(&mut self) -> Result<Node, Error> {
        let line: usize = self.curr.line;
        let btype: TokenType = self.curr.ttype;
        self.expect(btype)?;

        let l: Node = self.prev_expr.clone();
        self.ignore_ops = true;
        let r: Node = self.parse_expr()?.unwrap();

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
            if let Some(expr) = self.parse_expr()? {
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

