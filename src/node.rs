use crate::error::Error;

#[derive(Clone, Debug)]
pub enum DtypeVariant {
    Int,
    Char
}

#[derive(Clone, Debug)]
pub struct Dtype {
    variant: DtypeVariant,
    indirection: Vec<char>
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
    Param {
        name: String,
        dtype: Dtype
    },
    Vardef {
        name: String,
        value: Node,
        dtype: Dtype
    },
    Var {
        name: String,
        indirection: Vec<char>
    },
    Assign {
        l: Node,
        r: Node
    },
    If {
        cond: Node,
        body: Node
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub variant: Box<NodeVariant>,
    pub line: usize
}

impl Dtype {
    pub fn new(dtype: String, indirection: Vec<char>) -> Result<Self, Error> {
        Ok(Self { variant: Dtype::str2variant(dtype)?, indirection })
    }

    pub fn str2variant(s: String) -> Result<DtypeVariant, Error> {
        match s.as_str() {
            "int" => Ok(DtypeVariant::Int),
            "char" => Ok(DtypeVariant::Char),
            _ => Err(Error::new(format!("{} is not a valid data type.", s), 0))
        }
    }
}

impl Node {
    pub fn new(variant: NodeVariant, line: usize) -> Self {
        Self { variant: Box::new(variant), line }
    }
}

