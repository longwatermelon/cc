use crate::node::{Node, NodeVariant};

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

struct ScopeLayer {
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
    pub fn new(node: &Node, scope: &Scope) -> Self {
        let mut stack_offsets: Vec<i32> = Vec::new();
        let NodeVariant::Fdef { params, .. } = node.variant.as_ref() else { unreachable!() };

        let mut offset: i32 = 4;
        for param in params.iter().rev() {
            offset += param.dtype(scope).variant.num_bytes();
            stack_offsets.push(offset);
        }

        Self { node: node.clone(), param_stack_offsets: stack_offsets }
    }
}

impl ScopeLayer {
    fn new() -> Self {
        Self { vardefs: Vec::new(), stack_offset: 0 }
    }

    fn push_vardef(&mut self, v: CVardef) {
        self.vardefs.push(v);
    }
}

impl Scope {
    pub fn new() -> Self {
        Self { layers: vec![ScopeLayer::new()], fdefs: Vec::new() }
    }

    pub fn push_layer(&mut self) {
        self.layers.push(ScopeLayer::new());
    }

    pub fn pop_layer(&mut self) {
        self.layers.pop();
    }

    pub fn push_vardef(&mut self, n: &Node) {
        // self.layers must have len >= 1
        if let NodeVariant::Vardef { dtype, .. } = n.variant.as_ref() {
            self.layers.last_mut().unwrap().stack_offset -= dtype.variant.num_bytes();
            let offset: i32 = self.layers.last().unwrap().stack_offset;
            self.layers.last_mut().unwrap().push_vardef(CVardef::new(n, offset));
        } else {
            panic!("push_vardef received {:?}", n.variant);
        }
    }

    fn push_cvardef(&mut self, cv: &CVardef) {
        self.layers.last_mut().unwrap().push_vardef(cv.clone());
    }

    pub fn push_fdef(&mut self, n: &Node) {
        if let NodeVariant::Fdef {..} = n.variant.as_ref() {
            self.fdefs.push(CFdef::new(n, self));
        } else {
            panic!("push_fdef received {:?}", n.variant);
        }
    }

    /// Pushes vardefs into the current scope. Doesn't set them to any function args.
    pub fn push_fdef_params(&mut self, name: String) {
        let fdef: CFdef = self.find_fdef(name).unwrap().clone();
        let NodeVariant::Fdef { params, .. } = fdef.node.variant.as_ref() else { unreachable!() };
        for (i, param) in params.clone().iter().enumerate() {
            self.push_cvardef(&CVardef::new(&param, fdef.param_stack_offsets[i]));
        }
    }

    pub fn find_fdef(&self, name: String) -> Option<&CFdef> {
        for fdef in &self.fdefs {
            if let NodeVariant::Fdef { name: fname, .. } = fdef.node.variant.as_ref() {
                if fname.clone() == name {
                    return Some(fdef);
                }
            }
        }

        None
    }

    pub fn find_vardef(&self, name: String) -> Option<&CVardef> {
        for layer in &self.layers {
            for def in &layer.vardefs {
                if def.node.vardef_name() == name {
                    return Some(def);
                }
            }
        }

        None
    }
}

