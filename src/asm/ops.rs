use super::instruction::AsmArg;
use super::util;
use super::Gen;
use crate::cdefs::{CVardef, CStruct};
use crate::error::{Error, ErrorType};
use crate::lexer::TokenType;
use crate::node::{Dtype, DtypeVariant, Node, NodeVariant};
use crate::scope::Scope;

impl Gen {
    pub fn gen_binop(&mut self, n: &Node) -> Result<String, Error> {
        let NodeVariant::Binop { btype, l, r } = n.variant.as_ref() else { unreachable!() };
        match btype {
            TokenType::Dot => self.gen_memb_access(l, r),
            TokenType::Equal => self.asm_mov(AsmArg::Node(l), AsmArg::Node(r), true),
            TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Div => {
                self.asm_arithmetic(AsmArg::Node(l), AsmArg::Node(r), *btype)
            }
            TokenType::EqualCmp => self.gen_cmp(l, r, "je"),
            TokenType::NotEqual => self.gen_cmp(l, r, "jne"),
            TokenType::And | TokenType::Or => self.gen_andor(l, r, *btype),
            _ => panic!("[Gen::gen_binop] Binop {:?} not supported.", btype),
        }
    }

    pub fn gen_unop(&mut self, n: &Node) -> Result<String, Error> {
        let NodeVariant::Unop { utype, r } = n.variant.as_ref() else { unreachable!() };
        match utype {
            TokenType::Not => self.gen_not(r),
            TokenType::Amp => self.gen_addressof(r),
            TokenType::Star => self.gen_deref(r),
            _ => panic!("[Gen::gen_unop] Unop {:?} not supported.", utype),
        }
    }

    fn gen_memb_access(&mut self, l: &Node, r: &Node) -> Result<String, Error> {
        // Member access must be an identifier
        if !matches!(r.variant.as_ref(), NodeVariant::Var { .. }) {
            return Err(Error::new(ErrorType::StructMemberVarNonId(r), r.line));
        }

        // Only structs have member variables
        let dtype: Dtype = l.dtype(&self.scope)?;
        if !matches!(dtype.variant, DtypeVariant::Struct { .. }) {
            return Err(Error::new(ErrorType::PrimitiveMemberAccess(dtype), l.line));
        }

        // Get offset of member specified by r
        // First take care of any nesting before the last operand (a.b.c -> (a.b).c)
        fn nested_offset<'a>(n: &'a Node, scope: &'a Scope) -> Result<(i32, &'a CStruct), Error> {
            if let NodeVariant::Binop { l, r, .. } = n.variant.as_ref() {
                let (offset, sdef) = nested_offset(l, scope)?;
                Ok((
                    offset + sdef.offset_of(r.var_name().as_str(), r.line)?,
                    scope.find_struct_dtype(n.dtype(scope)?, n.line)?,
                ))
            } else {
                // n is a var type
                let offset: i32 = scope
                    .find_vardef(n.var_name().as_str(), n.line)?
                    .stack_offset;
                Ok((offset, scope.find_struct_dtype(n.dtype(scope)?, n.line)?))
            }
        }

        let (l_offset, sdef) = nested_offset(l, &self.scope)?;
        // For error message later
        let NodeVariant::Struct { name: sdef_name, .. } = sdef.node.variant.as_ref() else { unreachable!() };

        // Get offset of right operand relative to possibly nested expression's struct type
        let memb_name: String = r.var_name();
        let NodeVariant::Struct { fields, .. } = sdef.node.variant.as_ref() else { unreachable!() };
        let index: usize = fields
            .iter()
            .position(|x| x.vardef_name() == memb_name)
            .ok_or(Error::new(
                ErrorType::NonexistentStructMember(sdef_name.as_str(), memb_name.as_str()),
                l.line,
            ))?;

        let rel_offset: i32 = sdef.memb_stack_offsets[index];
        let memb_dtype: Dtype = fields[index].dtype(&self.scope)?;

        // mov register, member
        let offset: i32 = l_offset + rel_offset;
        let reg: String = memb_dtype.register('b', &self.scope)?;
        self.asm_mov(
            AsmArg::Register(&reg),
            AsmArg::Stack(&memb_dtype, offset),
            true,
        )
    }

    fn gen_cmp(&mut self, l: &Node, r: &Node, jmp: &str) -> Result<String, Error> {
        /*
            cmp l, r
            <zf conditional>
        */
        Ok(format!(
            "{}{}",
            self.asm_cmp(AsmArg::Node(l), AsmArg::Node(r))?,
            self.asm_zf_conditional(util::register('a', l, self)?.as_str(), jmp)
        ))
    }

    fn gen_andor(&mut self, l: &Node, r: &Node, op: TokenType) -> Result<String, Error> {
        let ar: String = util::register('a', l, self)?;
        let br: String = util::register('b', l, self)?;

        let zero_node: Node = Node::new(NodeVariant::Int { value: 0 }, l.line);
        let lcmp: String = format!(
            "{}{}",
            self.gen_cmp(l, &zero_node, "jne")?,
            self.asm_mov(
                AsmArg::Register(br.as_str()),
                AsmArg::Register(ar.as_str()),
                true
            )?,
        );

        let rcmp: String = self.gen_cmp(r, &zero_node, "jne")?;

        let asmop: String = format!(
            "\n\t; [andor]\n\t{} {}, {}\n\ttest {}, {}",
            match op {
                TokenType::And => "and",
                TokenType::Or => "or",
                _ => unreachable!(),
            },
            ar,
            br,
            ar,
            ar
        );
        let to_eax: String = self.asm_zf_conditional(ar.as_str(), "jnz");

        Ok(format!("{}{}{}{}", lcmp, rcmp, asmop, to_eax))
    }

    fn gen_not(&mut self, n: &Node) -> Result<String, Error> {
        let zero_node: Node = Node::new(NodeVariant::Int { value: 0 }, n.line);
        Ok(format!("\n\t; [not]{}", self.gen_cmp(n, &zero_node, "je")?))
    }

    fn gen_addressof(&mut self, n: &Node) -> Result<String, Error> {
        if !matches!(n.variant.as_ref(), NodeVariant::Var {..}) {
            return Err(Error::new(ErrorType::InvalidAddressof(n.variant.as_ref()), n.line));
        }

        let vardef: &CVardef = self.scope.find_vardef(n.var_name().as_str(), n.line)?;
        let offset: i32 = vardef.stack_offset;

        // lea rax, [rbp-(n offset)]
        Ok(format!("\n\t; [addressof]\n\tlea rax, [rbp{}]", offset))
    }

    fn gen_deref(&mut self, n: &Node) -> Result<String, Error> {
        /*
           ; Load n address into rax
           mov rax, [rbp-(n offset)]
           ; Dereference rax and store in rax (may be repeated for nested statements)
           mov rax, QWORD [rax]
           ; Store result in appropriately-sized register
           ; In this case if rax was an int* then eax would be used, and DWORD would be used for dereferencing
           mov eax, DWORD [rax]
        */

        if !matches!(n.variant.as_ref(), NodeVariant::Var {..} | NodeVariant::Unop {..}) {
            return Err(Error::new(ErrorType::InvalidDeref(n.variant.as_ref()), n.line));
        }

        let vardef: &CVardef = self.scope.find_vardef(n.var_name().as_str(), n.line)?;
        let offset: i32 = vardef.stack_offset;
        let stack_to_rax: String = format!("\n\tmov rax, {}", self.gen_stack_repr(&n.dtype(&self.scope)?, offset)?);

        let dtype: Dtype = n.dtype(&self.scope)?;
        // One deref left when moving to appropriately sized register
        // However the value passed is already the value being dereferenced
        // so these two effects cancel out, leaving dtype.nderefs
        let derefs: String = "\n\tmov rax, QWORD [rax]".repeat(dtype.nderefs);

        let rax_to_result: String = format!("\n\tmov {}, {} [rax]", dtype.register('a', &self.scope)?, dtype.deref(&self.scope)?);

        Ok(format!(
            "\n\t; [deref stack to rax]{}\n\t; [deref handle nested]{}
            \n\t; [deref rax to result]{}",
            stack_to_rax, derefs, rax_to_result
        ))
    }
}
