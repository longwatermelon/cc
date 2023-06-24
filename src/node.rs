use crate::error::Error;
use crate::lexer::TokenType;

#[derive(Clone, Debug)]
pub enum Dtype {
    Int,
    Char
}

impl Dtype {
    pub fn new(dtype: String) -> Result<Self, Error> {
        match dtype.as_str() {
            "int" => Ok(Dtype::Int),
            "char" => Ok(Dtype::Char),
            _ => Err(Error::new(format!("{} is not a valid data type.", dtype), 0))
        }
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
    Assign {
        l: Node,
        r: Node
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
    Ref {
        value: Node
    },
    Deref {
        value: Node
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
            NodeVariant::Ref { value } | NodeVariant::Deref { value } => value.var_name(),
            NodeVariant::Var { name } => name.clone(),
            _ => panic!()
        }
    }
}

