use crate::node::{Node, NodeVariant};
use crate::error::Error;

#[derive(Clone)]
pub struct CVardef {
    pub node: Node,
    pub stack_offset: i32
}

#[derive(Clone)]
pub struct CFdef {
    pub node: Node,
    pub param_stack_offsets: Vec<i32>
}

pub struct ScopeLayer {
    vardefs: Vec<CVardef>,
    stack_offset: i32
}

pub struct Scope {
    layers: Vec<ScopeLayer>,
    fdefs: Vec<CFdef>
}

impl CVardef {
    pub fn new(node: &Node, stack_offset: i32) -> Self {
        Self { node: node.clone(), stack_offset }
    }
}

impl CFdef {
    pub fn new(node: &Node, scope: &Scope) -> Result<Self, Error> {
        let mut stack_offsets: Vec<i32> = Vec::new();
        let NodeVariant::Fdef { params, .. } = node.variant.as_ref() else { unreachable!() };

        let mut offset: i32 = 16;
        for param in params.iter().rev() {
            stack_offsets.push(offset);
            offset += param.dtype(scope)?.variant.num_bytes();
        }

        Ok(Self { node: node.clone(), param_stack_offsets: stack_offsets })
    }
}

impl ScopeLayer {
    fn new() -> Self {
        Self { vardefs: Vec::new(), stack_offset: 0 }
    }

    fn push_vardef(&mut self, v: CVardef) {
        self.vardefs.push(v);
    }

    fn pop_vardef(&mut self) -> CVardef {
        self.vardefs.pop().unwrap()
    }
}

impl Scope {
    pub fn new() -> Self {
        Self { layers: vec![ScopeLayer::new()], fdefs: Vec::new() }
    }

    pub fn push_layer(&mut self) {
        self.layers.push(ScopeLayer::new());
    }

    pub fn push_layer_from(&mut self, layer: ScopeLayer) {
        self.layers.push(layer);
    }

    pub fn pop_layer(&mut self) -> ScopeLayer {
        self.layers.pop().unwrap()
    }

    /// Doesn't modify stack offset, uses self.stack_offset()
    pub fn push_vardef(&mut self, n: &Node, err_line: usize) -> Result<(), Error> {
        if self.find_vardef(n.vardef_name(), err_line).is_ok() {
            return Err(Error::new(format!("redefinition of variable {}", n.vardef_name()), n.line));
        }

        // self.layers must have len >= 1
        let offset: i32 = self.stack_offset();
        self.layers.last_mut().unwrap().push_vardef(CVardef::new(n, offset));

        Ok(())
    }

    pub fn pop_vardef(&mut self) -> CVardef {
        self.layers.last_mut().unwrap().pop_vardef()
    }

    pub fn push_cvardef(&mut self, cv: &CVardef) {
        self.layers.last_mut().unwrap().push_vardef(cv.clone());
    }

    pub fn push_fdef(&mut self, n: &Node) -> Result<(), Error> {
        let NodeVariant::Fdef { name: _, .. } = n.variant.as_ref() else { panic!("push_fdef received {:?}", n.variant) };
        self.fdefs.push(CFdef::new(n, self)?);

        Ok(())
    }

    pub fn find_fdef(&self, name: String, err_line: usize) -> Result<&CFdef, Error> {
        self.fdefs.iter().find(|&x| {
            let NodeVariant::Fdef { name: fname, .. } = x.node.variant.as_ref() else { unreachable!() };
            fname.clone() == name
        }).ok_or(Error::new(format!("function {} does not exist.", name), err_line))
    }

    pub fn find_vardef(&self, name: String, err_line: usize) -> Result<&CVardef, Error> {
        for layer in &self.layers {
            let result: Option<&CVardef> = layer.vardefs.iter().find(|&x|
                x.node.vardef_name() == name
            );

            if result.is_some() {
                return Ok(result.unwrap());
            }
        }

        Err(Error::new(format!("variable {} does not exist.", name), err_line))
    }

    pub fn stack_offset(&self) -> i32 {
        self.layers.last().unwrap().stack_offset
    }

    pub fn stack_offset_change(&mut self, delta: i32) {
        self.layers.last_mut().unwrap().stack_offset += delta;
    }

    pub fn stack_offset_change_n(&mut self, n: &Node) -> Result<(), Error> {
        self.stack_offset_change(-n.dtype(self)?.variant.num_bytes());
        Ok(())
    }
}

