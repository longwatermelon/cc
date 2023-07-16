use crate::error::Error;
use crate::asm::Gen;
use crate::node::{Node, Dtype};
use crate::lexer::TokenType;

pub enum AsmArg<'a> {
    Node(&'a Node),
    /// Full register name
    Register(&'a str),
    Stack(&'a Dtype, i32),
}

impl<'a> AsmArg<'a> {
    fn repr(&self, gen: &mut Gen) -> Result<String, Error> {
        match self {
            AsmArg::Node(n) => gen.gen_repr(n),
            AsmArg::Register(reg) => Ok(reg.to_string()),
            AsmArg::Stack(dtype, offset) => gen.stack_repr(&dtype, *offset),
        }
    }

    fn associated_register(&self, gen: &mut Gen, reg: char) -> Result<String, Error> {
        match self {
            AsmArg::Node(n) => n.dtype(&gen.scope)?.variant.register(reg, &gen.scope),
            AsmArg::Register(r) => Ok(r.to_string()),
            AsmArg::Stack(dtype, _) => dtype.variant.register(reg, &gen.scope),
        }
    }

    fn gen_expr_if_needed(&self, gen: &mut Gen) -> Result<String, Error> {
        if let AsmArg::Node(n) = self {
            gen.gen_expr(n)
        } else {
            Ok(String::new())
        }
    }
}

#[cfg(target_arch = "x86_64")]
impl Gen {
    pub fn mov(&mut self, dest: AsmArg, src: AsmArg) -> Result<String, Error> {
        let exprs: String = format!("{}{}",
            dest.gen_expr_if_needed(self)?,
            src.gen_expr_if_needed(self)?
        );

        // Get dest and src asm reprs
        let dest_repr: String = dest.repr(self)?;
        let src_repr: String = src.repr(self)?;

        if dest_repr == src_repr {
            return Ok(exprs);
        }

        // Avoid mem to mem by moving to a register first
        let reg: String = dest.associated_register(self, 'b')?;

        let src_to_dest: String = if dest_repr.contains('[') && src_repr.contains('[') {
            let src_to_reg: String = format!("\n\tmov {}, {}", reg, src_repr);
            let reg_to_dest: String = format!("\n\tmov {}, {}", dest_repr, reg);
            format!("{}{}", src_to_reg, reg_to_dest)
        } else {
            format!("\n\tmov {}, {}", dest_repr, src_repr)
        };

        Ok(format!("{}{}",
            exprs,
            src_to_dest
        ))
    }

    pub fn extend_stack(&self, nbytes: i32) -> String {
        format!("\n\tsub rsp, {}", nbytes)
    }

    /// Result in eax
    pub fn arithmetic(&mut self, a: AsmArg, b: AsmArg, op: TokenType) -> Result<String, Error> {
        let expr_a: String = a.gen_expr_if_needed(self)?;
        let expr_b: String = b.gen_expr_if_needed(self)?;

        let reg_a: String = a.associated_register(self, 'a')?;
        let reg_b: String = b.associated_register(self, 'b')?;

        let a_to_reg: String = self.mov(AsmArg::Register(&reg_a), a)?;
        let b_to_reg: String = self.mov(AsmArg::Register(&reg_b), b)?;

        Ok(format!(
            "{}{}{}{}\n\t{}",
            expr_a,
            expr_b,
            a_to_reg,
            b_to_reg,
            match op {
                TokenType::Plus => format!("add {}, {}", reg_a, reg_b),
                TokenType::Minus => format!("sub {}, {}", reg_a, reg_b),
                TokenType::Star => format!("mul {}", reg_b),
                TokenType::Div => format!("div {}", reg_b),
                _ => unreachable!(),
            }
        ))
    }
}

