use crate::error::{Error, ErrorType};
use crate::node::{Node, NodeVariant};
use crate::scope::Scope;

#[derive(Clone)]
pub struct CVardef {
    pub node: Node,
    /// Real offset
    pub stack_offset: i32,
}

impl CVardef {
    pub fn new(node: &Node, stack_offset: i32) -> Self {
        Self {
            node: node.clone(),
            stack_offset,
        }
    }
}

#[derive(Clone)]
pub struct CFdef {
    pub node: Node,
    /// Abs value offsets
    pub param_stack_offsets: Vec<i32>,
}

impl CFdef {
    pub fn new(node: &Node, scope: &Scope) -> Result<Self, Error> {
        let mut stack_offsets: Vec<i32> = Vec::new();
        let NodeVariant::Fdef { params, .. } = node.variant.as_ref() else { unreachable!() };

        let mut offset: i32 = 16;
        for param in params.iter() {
            stack_offsets.push(offset);
            offset += param.dtype(scope)?.num_bytes(scope)?;
        }

        Ok(Self {
            node: node.clone(),
            param_stack_offsets: stack_offsets,
        })
    }
}

#[derive(Clone)]
pub struct CStruct {
    pub node: Node,
    /// Abs value offsets
    pub memb_stack_offsets: Vec<i32>,
}

impl CStruct {
    pub fn new(node: &Node, scope: &Scope) -> Result<Self, Error> {
        let mut stack_offsets: Vec<i32> = Vec::new();
        let NodeVariant::Struct { fields, .. } = node.variant.as_ref() else { unreachable!() };

        let mut offset: i32 = 0;
        for field in fields.iter() {
            stack_offsets.push(offset);
            offset += field.dtype(scope)?.num_bytes(scope)?;
        }

        Ok(Self {
            node: node.clone(),
            memb_stack_offsets: stack_offsets,
        })
    }

    pub fn offset_of(&self, field_name: &str, err_line: usize) -> Result<i32, Error> {
        let NodeVariant::Struct { name, fields } = self.node.variant.as_ref() else { unreachable!() };
        let index: usize = fields
            .iter()
            .position(|x| x.vardef_name() == field_name)
            .ok_or(Error::new(
                ErrorType::NonexistentStructMember(name.as_str(), field_name),
                err_line,
            ))?;

        Ok(self.memb_stack_offsets[index])
    }
}
