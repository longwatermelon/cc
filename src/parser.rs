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

            if self.prev.ttype != TokenType::Rbrace {
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
        let n: Node = match self.curr.ttype {
            TokenType::Str => self.parse_str()?,
            TokenType::Int => self.parse_int()?,
            TokenType::Id => self.parse_id()?,
            TokenType::Star | TokenType::Amp => self.parse_var()?,
            TokenType::Lbrace => {
                self.expect(TokenType::Lbrace)?;
                let node = self.parse()?;
                self.expect(TokenType::Rbrace)?;
                node
            },
            _ => return Ok(None)
        };

        self.prev_expr = n.clone();

        if !self.ignore_ops {
            if self.curr.ttype == TokenType::Plus {
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

            println!("{}", self.curr.value);

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
        // Start on indirection, or name if indirection is inapplicable
        // begin_tok is either a type or something meaningless, only used for checking if vardef
        let begin_tok: String = self.curr.value.clone();
        let next_type: TokenType = self.lexer.peek(1)?.ttype;
        // terrible hack
        if self.curr.ttype == TokenType::Id && (
            next_type == TokenType::Id || 
            next_type == TokenType::Star ||
            next_type == TokenType::Amp ||
            next_type == TokenType::Equal
        ) {
            self.expect(TokenType::Id)?;
        }

        let indirection: Vec<char> = self.parse_indirection();
        let name: String = self.curr.value.clone();
        let line: usize = self.curr.line;

        let node_var: Node = Node::new(NodeVariant::Var { name: name.clone(), indirection: indirection.clone() }, line);

        // Vardefs always start with a type
        if Dtype::str2variant(begin_tok.clone()).is_ok() {
            self.parse_vardef(Dtype::new(begin_tok, indirection)?)
        } else {
            if !indirection.is_empty() {
                self.expect(TokenType::Id)?;
            }

            match self.curr.ttype {
                TokenType::Equal => {
                    self.parse_assign(indirection)
                },
                _ => {
                    if indirection.is_empty() {
                        self.expect(TokenType::Id)?;
                    }
                    Ok(node_var)
                }
            }
        }
    }

    fn parse_indirection(&mut self) -> Vec<char> {
        let mut res: Vec<char> = Vec::new();
        while self.curr.ttype == TokenType::Star || self.curr.ttype == TokenType::Amp {
            res.push(self.curr.value.chars().nth(0).unwrap());
            self.expect(self.curr.ttype).unwrap();
        }

        res
    }

    fn parse_vardef(&mut self, dtype: Dtype) -> Result<Node, Error> {
        let name: String = self.curr.value.clone();
        let line: usize = self.curr.line;
        self.expect(TokenType::Id)?;

        match self.curr.ttype {
            TokenType::Equal => {
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
            },
            TokenType::Lparen => {
                self.parse_fdef(dtype)
            },
            _ => Ok(Node::new(NodeVariant::Param { name, dtype }, line))
        }
    }

    fn parse_assign(&mut self, indirection: Vec<char>) -> Result<Node, Error> {
        let line: usize = self.curr.line;
        let l: Node = Node::new(NodeVariant::Var { name: self.prev.value.clone(), indirection }, line);
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

        if self.curr.ttype == TokenType::Plus {
            self.prev_expr = n;
            self.parse_binop()
        } else {
            Ok(n)
        }
        // struct Node *node = node_alloc(NODE_BINOP);
        // node->error_line = parser->curr_tok->line_num;
        // node->op_stack_offset = -parser->stack_size;
        // parser->stack_size += 8;

        // node->op_type = parser->curr_tok->binop_type;
        // parser_advance(parser, 1);

        // node->op_l = parser->prev_node;
        // node->op_r = parser_parse_expr(parser, true);

        // if (parser->curr_tok->type == TOKEN_BINOP)
        // {
        //     parser->prev_node = node;
        //     struct Node *root = parser_parse_binop(parser);

        //     return root;
        // }
        // else
        // {
        //     return node;
        // }
    }
}

