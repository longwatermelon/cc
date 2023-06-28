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

#[derive(Clone)]
pub struct CStruct {
    pub node: Node,
    pub memb_stack_offsets: Vec<i32>
}

pub struct ScopeLayer {
    vardefs: Vec<CVardef>,
    stack_offset: i32
}

pub struct Scope {
    layers: Vec<ScopeLayer>,
    fdefs: Vec<CFdef>,
    structs: Vec<CStruct>
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

impl CStruct {
    pub fn new(node: &Node, scope: &Scope) -> Result<Self, Error> {
        let mut stack_offsets: Vec<i32> = Vec::new();
        let NodeVariant::Struct { fields, .. } = node.variant.as_ref() else { unreachable!() };

        let mut offset: i32 = 16;
        for field in fields.iter() {
            stack_offsets.push(offset);
            offset += field.dtype(scope)?.variant.num_bytes();
        }

        Ok(Self { node: node.clone(), memb_stack_offsets: stack_offsets })
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
        Self { layers: vec![ScopeLayer::new()], fdefs: Vec::new(), structs: Vec::new() }
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
        if self.find_vardef(&n.vardef_name(), err_line).is_ok() {
            return Err(Error::new(format!("redefinition of variable '{}'", n.vardef_name()), n.line));
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
        let NodeVariant::Fdef { name: fname, params, rtype, .. } = n.variant.as_ref() else { panic!("push_fdef received {:?}", n.variant) };

        // Check if fdef exists
        if let Ok(fdef) = self.find_fdef(fname, n.line) {
            let NodeVariant::Fdef { body, params: orig_params, rtype: orig_rtype, .. } = fdef.node.variant.as_ref() else { unreachable!() };

            // If declaration, replace. Otherwise it's a redef error
            if matches!(body.variant.as_ref(), NodeVariant::Noop) {
                if params.len() != orig_params.len() || rtype.variant != orig_rtype.variant {
                    return Err(Error::new(format!("definition of '{}' does not align with its declaration.", fname), n.line));
                }

                // Keep all fdefs with name != fname
                self.fdefs.retain(|x| {
                    let NodeVariant::Fdef { name, .. } = x.node.variant.as_ref() else { unreachable!() };
                    name != fname
                });
            } else {
                return Err(Error::new(format!("redefinition of function '{}'.", fname), n.line));
            }
        }

        self.fdefs.push(CFdef::new(n, self)?);
        Ok(())
    }

    pub fn push_struct(&mut self, n: &Node) -> Result<(), Error> {
        let NodeVariant::Struct { name, .. } = n.variant.as_ref() else { panic!("push_fdef received {:?}", n.variant) };

        // Check if fdef exists
        if let Ok(st) = self.find_struct(name, n.line) {
            let NodeVariant::Struct { name: orig_name, fields: orig_fields } = st.node.variant.as_ref() else { unreachable!() };

            // If declaration, replace. Otherwise it's a redef error
            if orig_fields.is_empty() {
                // Remove original struct
                self.fdefs.retain(|x| {
                    let NodeVariant::Struct { name: sname, .. } = x.node.variant.as_ref() else { unreachable!() };
                    sname != name
                });
            } else {
                return Err(Error::new(format!("redefinition of struct '{}'.", orig_name), n.line));
            }
        }

        self.structs.push(CStruct::new(n, self)?);
        Ok(())
    }

    pub fn find_fdef(&self, name: &str, err_line: usize) -> Result<&CFdef, Error> {
        self.fdefs.iter().find(|&x| {
            let NodeVariant::Fdef { name: fname, .. } = x.node.variant.as_ref() else { unreachable!() };
            fname == name
        }).ok_or(Error::new(format!("function '{}' does not exist.", name), err_line))
    }

    pub fn find_struct(&self, name: &str, err_line: usize) -> Result<&CStruct, Error> {
        self.structs.iter().find(|&x| {
            let NodeVariant::Struct { name: orig_name, .. } = x.node.variant.as_ref() else { unreachable!() };
            name == orig_name
        }).ok_or(Error::new(format!("struct '{}' does not exist.", name), err_line))
    }

    pub fn find_vardef(&self, name: &str, err_line: usize) -> Result<&CVardef, Error> {
        for layer in &self.layers {
            let result: Option<&CVardef> = layer.vardefs.iter().find(|&x|
                x.node.vardef_name() == name
            );

            if let Some(res) = result {
                return Ok(res);
            }
        }

        Err(Error::new(format!("variable '{}' does not exist.", name), err_line))
    }

    pub fn stack_offset(&self) -> i32 {
        self.layers.last().unwrap().stack_offset
    }

    pub fn stack_offset_change(&mut self, delta: i32) {
        self.layers.last_mut().unwrap().stack_offset += delta;
    }

    pub fn stack_offset_change_n(&mut self, n: &Node, direction: i32) -> Result<(), Error> {
        self.stack_offset_change(direction * n.dtype(self)?.variant.num_bytes());
        Ok(())
    }
}

