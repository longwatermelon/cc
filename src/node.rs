#[derive(Clone, Debug)]
pub enum Dtype {
    Int
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
    Vardef {
        name: String,
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
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub variant: Box<NodeVariant>,
    pub line: usize
}

impl Dtype {
    pub fn from_str(dtype: String) -> Self {
        match dtype.as_str() {
            "int" => Dtype::Int,
            _ => panic!("{} is not a valid dtype", dtype)
        }
    }
}

impl Node {
    pub fn new(variant: NodeVariant, line: usize) -> Self {
        Self { variant: Box::new(variant), line }
    }
}

