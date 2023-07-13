use super::Gen;
use crate::error::Error;
use crate::node::{Node, NodeVariant, Dtype, DtypeVariant};
use crate::lexer::TokenType;
use crate::cdefs::{CStruct, CVardef};

impl Gen {
    pub fn gen_binop(&mut self, n: &Node) -> Result<String, Error> {
        let NodeVariant::Binop { btype, l, r } = n.variant.as_ref() else { unreachable!() };
        match btype {
            TokenType::Dot => self.gen_memb_access(l, r),
            _ => panic!("[Gen::gen_binop] Binop {:?} not supported.", btype),
        }
    }

    pub fn gen_memb_access(&mut self, l: &Node, r: &Node) -> Result<String, Error> {
        // id.id format is required
        if !matches!(r.variant.as_ref(), NodeVariant::Var {..}) {
            return Err(Error::new(String::from("Member variable must be an identifier."), r.line));
        }

        // Only structs have member variables
        let dtype: Dtype = l.dtype(&self.scope)?;
        if !matches!(dtype.variant, DtypeVariant::Struct {..}) {
            return Err(Error::new(format!("Dtype {:?} does not have member variables.", dtype.variant), l.line));
        }

        // Find struct
        let DtypeVariant::Struct { name } = &dtype.variant else { unreachable!() };
        let sdef: &CStruct = self.scope.find_struct(name.as_str(), l.line)?;

        // Get offset of member specified by r
        let memb_name: String = r.var_name();
        let NodeVariant::Struct { fields, .. } = sdef.node.variant.as_ref() else { unreachable!() };
        let index: usize = fields.iter()
                            .position(|x| x.vardef_name() == memb_name)
                            .ok_or(
                                Error::new(
                                    format!(
                                        "Struct '{}' has no member '{}'.",
                                        name, memb_name
                                    ), l.line
                                )
                            )?;
        println!("{:?}", sdef.memb_stack_offsets);
        let offset: i32 = sdef.memb_stack_offsets[index];
        let memb_dtype: Dtype = fields[index].dtype(&self.scope)?;

        // mov register, member
        let vardef: &CVardef = self.scope.find_vardef(l.var_name().as_str(), l.line)?;
        println!("{}", vardef.stack_offset);
        let offset: i32 = vardef.stack_offset + offset;
        Ok(format!(
            "\n\tmov {}, {}",
            memb_dtype.variant.register("bx", &self.scope)?,
            self.stack_repr(&memb_dtype, offset)?
        ))
    }
}

