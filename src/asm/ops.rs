use super::Gen;
use crate::error::Error;
use crate::node::{Node, NodeVariant, Dtype, DtypeVariant};
use crate::lexer::TokenType;
use crate::cdefs::CStruct;
use crate::scope::Scope;

impl Gen {
    pub fn gen_binop(&mut self, n: &Node) -> Result<String, Error> {
        let NodeVariant::Binop { btype, l, r } = n.variant.as_ref() else { unreachable!() };
        match btype {
            TokenType::Dot => self.gen_memb_access(l, r),
            TokenType::Equal => self.gen_assign(l, r),
            _ => panic!("[Gen::gen_binop] Binop {:?} not supported.", btype),
        }
    }

    pub fn gen_memb_access(&mut self, l: &Node, r: &Node) -> Result<String, Error> {
        // Member access must be an identifier
        if !matches!(r.variant.as_ref(), NodeVariant::Var {..}) {
            return Err(Error::new(String::from("Member variable must be an identifier."), r.line));
        }

        // Only structs have member variables
        let dtype: Dtype = l.dtype(&self.scope)?;
        if !matches!(dtype.variant, DtypeVariant::Struct {..}) {
            return Err(Error::new(format!("Dtype {:?} does not have member variables.", dtype.variant), l.line));
        }

        // Get offset of member specified by r
        // First take care of any nesting before the last operand (a.b.c -> (a.b).c)
        fn nested_offset<'a>(n: &'a Node, scope: &'a Scope) -> Result<(i32, &'a CStruct), Error> {
            if let NodeVariant::Binop { l, r, .. } = n.variant.as_ref() {
                let (offset, sdef) = nested_offset(l, scope)?;
                Ok((offset + sdef.offset_of(r.var_name().as_str(), r.line)?, scope.find_struct_dtype(n.dtype(scope)?, n.line)?))
            } else {
                // n is a var type
                let offset: i32 = scope.find_vardef(n.var_name().as_str(), n.line)?.stack_offset;
                Ok((offset, scope.find_struct_dtype(n.dtype(scope)?, n.line)?))
            }
        }

        let (l_offset, sdef) = nested_offset(l, &self.scope)?;
        // For error message later
        let NodeVariant::Struct { name: sdef_name, .. } = sdef.node.variant.as_ref() else { unreachable!() };

        // Get offset of right operand relative to possibly nested expression's struct type
        let memb_name: String = r.var_name();
        let NodeVariant::Struct { fields, .. } = sdef.node.variant.as_ref() else { unreachable!() };
        let index: usize = fields.iter()
                            .position(|x| x.vardef_name() == memb_name)
                            .ok_or(
                                Error::new(
                                    format!(
                                        "Struct '{}' has no member '{}'.",
                                        sdef_name, memb_name
                                    ), l.line
                                )
                            )?;

        let rel_offset: i32 = sdef.memb_stack_offsets[index];
        let memb_dtype: Dtype = fields[index].dtype(&self.scope)?;

        // mov register, member
        let offset: i32 = l_offset + rel_offset;
        Ok(self.mov(
            &memb_dtype.variant.register("bx", &self.scope)?,
            &self.stack_repr(&memb_dtype, offset)?
        ))
    }

    pub fn gen_assign(&mut self, l: &Node, r: &Node) -> Result<String, Error> {
        let reg: String = l.dtype(&self.scope)?.variant.register("bx", &self.scope)?;
        let l_repr: String = self.gen_repr(l)?;
        let r_repr: String = self.gen_repr(r)?;

        let r_to_reg: String = self.mov(&reg, &r_repr);
        let reg_to_l: String = self.mov(&l_repr, &reg);

        Ok(format!("{}{}", r_to_reg, reg_to_l))
    }
}

