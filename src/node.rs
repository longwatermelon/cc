use crate::error::Error;
use crate::lexer::TokenType;
use crate::scope::Scope;

#[derive(Clone, Debug, PartialEq)]
pub enum DtypeVariant {
    Int,
    Char,
    Void,
    Struct { name: String }
}

#[derive(Clone, Debug)]
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

    pub fn num_bytes(&self) -> i32 {
        match self {
            DtypeVariant::Int => 4,
            DtypeVariant::Char => 1,
            DtypeVariant::Void => 0,
            DtypeVariant::Struct {..} => todo!()
        }
    }

    pub fn deref(&self) -> String {
        match self.num_bytes() {
            1 => "BYTE",
            4 => "DWORD",
            8 => "QWORD",
            _ => panic!("DtypeVariant::deref invalid size of {}", self.num_bytes())
        }.to_string()
    }

    pub fn register(&self, suffix: &str) -> String {
        match self.num_bytes() {
            1 | 4 => "e",
            8 => "r",
            _ => panic!("DtypeVariant::register invalid size of {}", self.num_bytes())
        }.to_string() + suffix
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
        values: Vec<Node>
    },
    Str {
        value: String
    },
    Int {
        value: i32
    },
    Char {
        value: char
    },
    Fcall {
        name: String,
        args: Vec<Node>
    },
    Fdef {
        name: String,
        params: Vec<Node>,
        body: Node,
        rtype: Dtype
    },
    Vardef {
        var: Node,
        value: Node,
        dtype: Dtype
    },
    Var {
        name: String
    },
    If {
        cond: Node,
        body: Node
    },
    Return {
        value: Node
    },
    Binop {
        btype: TokenType,
        l: Node,
        r: Node
    },
    Unop {
        utype: TokenType,
        r: Node
    },
    Struct {
        name: String,
        fields: Vec<Node>
    },
    For {
        init: Node,
        cond: Node,
        inc: Node,
        body: Node
    },
    While {
        cond: Node,
        body: Node
    },
    InitList {
        dtype: Dtype,
        fields: Vec<(String, Node)>
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
                _ => panic!("{:?} doesn't have a dtype.", self.variant)
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

