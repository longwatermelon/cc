use crate::error::Error;
use crate::lexer::TokenType;

#[derive(Clone, Debug)]
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
    pub fn new(dtype: String) -> Result<Self, Error> {
        match dtype.as_str() {
            "int" => Ok(DtypeVariant::Int),
            "char" => Ok(DtypeVariant::Char),
            "void" => Ok(DtypeVariant::Void),
            "struct" => Ok(DtypeVariant::Struct { name: String::new() }),
            _ => Err(Error::new(format!("{} is not a valid data type.", dtype), 0))
        }
    }
}

impl Dtype {
    pub fn new(dtype: String) -> Result<Self, Error> {
        Ok(Self { variant: DtypeVariant::new(dtype)?, memops: Vec::new() })
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

    pub fn var_name(&self) -> String {
        match self.variant.as_ref() {
            NodeVariant::Unop { r, .. } => r.var_name(),
            NodeVariant::Var { name } => name.clone(),
            _ => panic!()
        }
    }
}

