use crate::node::{Node, NodeVariant};

#[derive(Clone)]
pub struct CVardef {
    pub node: Node,
    pub stack_offset: i32
}

struct ScopeLayer {
    vardefs: Vec<CVardef>,
    stack_offset: i32
}

pub struct Scope {
    layers: Vec<ScopeLayer>,
    fdefs: Vec<Node>
}

impl CVardef {
    pub fn new(node: &Node, stack_offset: i32) -> Self {
        Self { node: node.clone(), stack_offset }
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

    pub fn push_fdef(&mut self, n: &Node) {
        if let NodeVariant::Fdef {..} = n.variant.as_ref() {
            self.fdefs.push(n.clone());
        } else {
            panic!("push_fdef received {:?}", n.variant);
        }
    }

    pub fn mut_vardef(&mut self, name: String) -> Option<&mut CVardef> {
        for layer in &mut self.layers {
            for def in &mut layer.vardefs {
                if def.node.vardef_name() == name {
                    return Some(def);
                }
            }
        }

        None
    }

    pub fn find_fdef(&self, name: String) -> Option<&Node> {
        for fdef in &self.fdefs {
            if let NodeVariant::Fdef { name: fname, .. } = fdef.variant.as_ref() {
                if fname.clone() == name {
                    return Some(fdef);
                }
            }
        }

        None
    }
}

