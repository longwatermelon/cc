use crate::error::Error;
use crate::lexer::TokenType;
use crate::scope::Scope;
use crate::cdefs::{CVardef, CStruct};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum DtypeVariant {
    Int,
    Char,
    Void,
    Struct { name: String }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dtype {
    pub variant: DtypeVariant,
    pub memops: Vec<char>
}

impl DtypeVariant {
    /// Does not fill out enum variant fields, only determines the enum variant type
    pub fn new(dtype: &str) -> Result<Self, Error> {
        match dtype {
            "int" => Ok(DtypeVariant::Int),
            "char" => Ok(DtypeVariant::Char),
            "void" => Ok(DtypeVariant::Void),
            "struct" => Ok(DtypeVariant::Struct { name: String::new() }),
            _ => Err(Error::new(format!("{} is not a valid data type.", dtype), 0))
        }
    }

    pub fn num_bytes(&self, scope: &Scope) -> Result<i32, Error> {
        Ok(
            match self {
                DtypeVariant::Int => 4,
                DtypeVariant::Char => 1,
                DtypeVariant::Void => 0,
                DtypeVariant::Struct { name } => {
                    let NodeVariant::Struct { fields, .. } = scope.find_struct(name.as_str(), 0)?
                                                                .node.variant.as_ref() else { unreachable!() };
                    let mut sum: i32 = 0;
                    for field in fields {
                        sum += field.dtype(scope)?.variant.num_bytes(scope)?;
                    }

                    sum
                },
            }
        )
    }

    pub fn deref(&self, scope: &Scope) -> Result<String, Error> {
        Ok(
            match self.num_bytes(scope)? {
                1 => "BYTE",
                4 => "DWORD",
                8 => "QWORD",
                _ => panic!("[DtypeVariant::deref] invalid size of {}", self.num_bytes(scope)?)
            }.to_string()
        )
    }

    pub fn register(&self, suffix: &str, scope: &Scope) -> Result<String, Error> {
        Ok(
            match self.num_bytes(scope)? {
                1 | 4 => "e",
                8 => "r",
                _ => panic!("[DtypeVariant::register] invalid size of {}", self.num_bytes(scope)?)
            }.to_string() + suffix
        )
    }
}

impl fmt::Display for DtypeVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            DtypeVariant::Int => "int".to_string(),
            DtypeVariant::Char => "char".to_string(),
            DtypeVariant::Void => "void".to_string(),
            DtypeVariant::Struct { name } => format!("struct {}", name)
        })
    }
}

impl Dtype {
    pub fn new(dtype: &str) -> Result<Self, Error> {
        Ok(Self { variant: DtypeVariant::new(dtype)?, memops: Vec::new() })
    }

    pub fn from_fields(variant: DtypeVariant) -> Self {
        Self { variant, memops: Vec::new() }
    }

    pub fn from_fields_memops(variant: DtypeVariant, memops: Vec<char>) -> Self {
        Self { variant, memops }
    }
}

#[derive(Clone, Debug)]
pub enum NodeVariant {
    Noop,
    Cpd {
        values: Vec<Node>,
    },
    Str {
        value: String,
    },
    Int {
        value: i32,
    },
    Char {
        value: char,
    },
    Fcall {
        name: String,
        args: Vec<Node>,
    },
    Fdef {
        name: String,
        params: Vec<Node>,
        body: Node,
        rtype: Dtype,
    },
    Vardef {
        var: Node,
        value: Node,
        dtype: Dtype,
    },
    Var {
        name: String,
    },
    If {
        cond: Node,
        body: Node,
    },
    Return {
        value: Node,
    },
    Binop {
        btype: TokenType,
        l: Node,
        r: Node,
    },
    Unop {
        utype: TokenType,
        r: Node,
    },
    Struct {
        name: String,
        /// Only vardefs
        fields: Vec<Node>,
    },
    For {
        init: Node,
        cond: Node,
        inc: Node,
        body: Node,
    },
    While {
        cond: Node,
        body: Node,
    },
    InitList {
        dtype: Dtype,
        fields: Vec<(String, Node)>,
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub variant: Box<NodeVariant>,
    pub line: usize
}

impl Node {
    pub fn new(variant: NodeVariant, line: usize) -> Self {
        Self { variant: Box::new(variant), line }
    }

    pub fn dtype(&self, scope: &Scope) -> Result<Dtype, Error> {
        Ok(
            match self.variant.as_ref() {
                NodeVariant::Str {..} => Dtype::from_fields_memops(DtypeVariant::Char, vec!['*']),
                NodeVariant::Int {..} => Dtype::from_fields(DtypeVariant::Int),
                NodeVariant::Char {..} => Dtype::from_fields(DtypeVariant::Char),
                NodeVariant::Fcall { name, .. } => scope.find_fdef(name, self.line)?.node.dtype(scope)?,
                NodeVariant::Fdef { rtype, .. } => rtype.clone(),
                NodeVariant::Vardef { dtype, .. } => dtype.clone(),
                NodeVariant::Var { name } => scope.find_vardef(name, self.line)?.node.dtype(scope)?,
                NodeVariant::InitList { dtype, .. } => dtype.clone(),
                // TODO r is a var type and when vardef lookup is
                // done for struct member access r is not found
                NodeVariant::Binop { l, r, btype } => {
                    // For struct member access, r.dtype will look for a variable
                    // with the same name as the struct member, which doesn't exist.
                    // If right operand is a binop, find the dtype of that. Otherwise,
                    // right operand has to be a var, so find the associated struct
                    // and get the member variable dtype.
                    if *btype == TokenType::Dot {
                        // Right operand is binop
                        if matches!(r.variant.as_ref(), NodeVariant::Binop {..}) {
                            r.dtype(scope)?
                        } else { // Right operand is var
                            let NodeVariant::Var { name, .. } = r.variant.as_ref() else { unreachable!() };
                            let vardef: &CVardef = scope.find_vardef(l.var_name().as_str(), l.line)?;
                            let DtypeVariant::Struct { name: struct_name } = vardef.node.dtype(scope)?.variant
                                                                             else { unreachable!() };
                            let sdef: &CStruct = scope.find_struct(struct_name.as_str(), self.line)?;

                            // Find relevant struct field
                            let NodeVariant::Struct { fields, .. } = sdef.node.variant.as_ref()
                                                                     else { unreachable!() };
                            let field: &Node = fields.iter().find(|&x| {
                                name == x.vardef_name().as_str()
                            }).ok_or(Error::new(format!("No member variable '{}' in struct '{}'.", name, struct_name), self.line))?;

                            field.dtype(scope)?
                        }
                    } else {
                        r.dtype(scope)?
                    }
                },
                _ => panic!("{:?} doesn't have a dtype.", self.variant)
            }
        )
    }

    /// For var / vardef, everything else will be returned as is.
    pub fn strip<'a>(&'a self, scope: &'a Scope) -> Result<&'a Node, Error> {
        Ok(
            match self.variant.as_ref() {
                NodeVariant::Var { name } => scope.find_vardef(name.as_str(), self.line)?.node.strip(scope)?,
                NodeVariant::Vardef { value, .. } => value.strip(scope)?,
                _ => self,
            }
        )
    }

    pub fn var_name(&self) -> String {
        match self.variant.as_ref() {
            NodeVariant::Unop { r, .. } => r.var_name(),
            NodeVariant::Var { name } => name.clone(),
            _ => panic!("var_name received {:?}", self.variant)
        }
    }

    pub fn vardef_name(&self) -> String {
        match self.variant.as_ref() {
            NodeVariant::Vardef { var, .. } => var.var_name(),
            _ => panic!("vardef_name received {:?}", self.variant)
        }
    }
}

