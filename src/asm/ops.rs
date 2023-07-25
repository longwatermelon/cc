use super::Gen;
use super::instruction::AsmArg;
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
            TokenType::Equal => self.mov(AsmArg::Node(l), AsmArg::Node(r)),
            TokenType::Plus |
            TokenType::Minus |
            TokenType::Star |
            TokenType::Div => self.arithmetic(AsmArg::Node(l), AsmArg::Node(r), *btype),
            TokenType::EqualCmp => self.gen_cmp(l, r, *btype),
            _ => panic!("[Gen::gen_binop] Binop {:?} not supported.", btype),
        }
    }

    pub fn gen_memb_access(&mut self, l: &Node, r: &Node) -> Result<String, Error> {
        // Member access must be an identifier
        if !matches!(r.variant.as_ref(), NodeVariant::Var {..}) {
            return Err(Error::new(format!("Member variable must be an identifier. Received: '{:?}'", r), r.line));
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
        let reg: String = memb_dtype.variant.register('b', &self.scope)?;
        self.mov(AsmArg::Register(&reg), AsmArg::Stack(&memb_dtype, offset))
    }

    /// Stores result in eax
    fn gen_cmp(&mut self, l: &Node, r: &Node, _op: TokenType) -> Result<String, Error> {
        // Cmp l, r
        // If equal, jump to set eax to 1
        // If not equal, don't jump until eax is set to 0
        // After eax is set to 0, jump across eax set to 1
        // .Lx+1 is the end
        /*
            cmp l, r
            je .Lx
            mov eax, 0
            jmp .Lx+1
            .Lx:
                mov eax, 1
            .Lx+1:
        */
        let cmp_je: String = format!("{}\n\tje .L{}",
            self.cmp(AsmArg::Node(l), AsmArg::Node(r))?,
            self.label
        );
        self.label += 1;

        let reg: String = l.dtype(&self.scope)?.variant.register('a', &self.scope)?;
        let when_false: String = format!("\n\tmov {}, 0\n\tjmp .L{}",
            reg, self.label
        );
        self.label += 1;

        let labels: String = format!(
            "\n.L{}:\n\tmov {}, 1\n.L{}:",
            self.label - 2,
            reg,
            self.label - 1,
        );

        Ok(format!("{}{}{}",
            cmp_je,
            when_false,
            labels,
        ))
    }
}

