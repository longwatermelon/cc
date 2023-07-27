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
    pub fn mov(&mut self, dest: AsmArg, src: AsmArg, gen_exprs: bool) -> Result<String, Error> {
        let exprs: String = if gen_exprs {
            format!("{}{}",
                dest.gen_expr_if_needed(self)?,
                src.gen_expr_if_needed(self)?
            )
        } else { String::new() };

        // Get dest and src asm reprs
        let dest_repr: String = dest.repr(self)?;
        let src_repr: String = src.repr(self)?;

        if dest_repr == src_repr {
            return Ok(exprs);
        }

        let src_to_dest: String = if dest_repr.contains('[') && src_repr.contains('[') {
            // Avoid mem to mem by moving to a register first
            let reg: String = dest.associated_register(self, 'b')?;
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

    /// Result in eax
    pub fn cmp(&mut self, a: AsmArg, b: AsmArg) -> Result<String, Error> {
        let exprs: String = format!(
            "{}{}",
            a.gen_expr_if_needed(self)?,
            b.gen_expr_if_needed(self)?,
        );

        let a_repr: String = a.repr(self)?;
        let b_repr: String = b.repr(self)?;

        Ok(format!("{}{}",
            exprs,
            format!("\n\tcmp {}, {}", a_repr, b_repr)
        ))
    }

    pub fn extend_stack(&self, nbytes: i32) -> String {
        format!("\n\tsub rsp, {}", nbytes)
    }

    /// a and b should be Nodes
    pub fn arithmetic(&mut self, a: AsmArg, b: AsmArg, op: TokenType) -> Result<String, Error> {
        // let expr_a: String = a.gen_expr_if_needed(self)?;
        // let expr_b: String = b.gen_expr_if_needed(self)?;

        let reg_a: String = a.associated_register(self, 'a')?;
        let reg_b: String = b.associated_register(self, 'b')?;

        let AsmArg::Node(na) = a else { unreachable!() };
        let AsmArg::Node(nb) = b else { unreachable!() };
        let a_expr: String = a.gen_expr_if_needed(self)?;
        self.scope.stack_offset_change_n(na, -1)?;
        let aoffset: i32 = self.scope.stack_offset();
        let a_to_stack: String = self.gen_stack_push(na)?;

        let b_expr: String = b.gen_expr_if_needed(self)?;
        self.scope.stack_offset_change_n(nb, -1)?;
        let boffset: i32 = self.scope.stack_offset();
        let b_to_stack: String = self.gen_stack_push(nb)?;

        let astack_to_reg: String = self.mov(
            AsmArg::Register(reg_a.as_str()),
            AsmArg::Stack(&na.dtype(&self.scope)?, aoffset),
            false
        )?;
        let bstack_to_reg: String = self.mov(
            AsmArg::Register(reg_b.as_str()),
            AsmArg::Stack(&nb.dtype(&self.scope)?, boffset),
            false
        )?;

        Ok(format!(
            "\n\t; [arithmetic] a expr{}\n\t; [arithmetic] a to stack{}\n\t; [arithmetic] b expr{}
             \n\t; [arithmetic] b to stack{}\n\t; [arithmetic] astack to reg{}\n\t; [arithmetic] bstack to reg{}
             \n\t{}\n\t; [arithmetic] end",
            a_expr,
            a_to_stack,
            b_expr,
            b_to_stack,
            astack_to_reg,
            bstack_to_reg,
            match op {
                TokenType::Plus => format!("add {}, {}", reg_a, reg_b),
                TokenType::Minus => format!("sub {}, {}", reg_a, reg_b),
                TokenType::Star => format!("mul {}", reg_b),
                TokenType::Div => format!("div {}", reg_b),
                _ => unreachable!(),
            },
        ))
    }

    /// Depending on the zero flag ZF, set eax to either 1 or 0.
    /// For example, given the operation cmp a, b, assuming param jmp = je:
    /// * If a == b, zf_conditional sets eax to 1
    /// * If a != b, zf_conditional sets eax to 0
    /// jmp = je, jne, etc.
    /// If you are comparing with a zero node, you most likely want jmp = jne.
    pub fn zf_conditional(&mut self, result_reg: &str, jmp: &str) -> String {
        /*
            je .Lx
            mov eax, 0
            jmp .Lx+1
            .Lx:
                mov eax, 1
            .Lx+1:
        */
        let jump: String = format!("\n\t{} .L{}", jmp, self.label);
        self.label += 1;

        let when_false: String = format!("\n\tmov {}, 0\n\tjmp .L{}",
            result_reg, self.label
        );
        self.label += 1;

        let labels: String = format!(
            "\n.L{}:\n\tmov {}, 1\n.L{}:",
            self.label - 2,
            result_reg,
            self.label - 1,
        );

        format!("{}{}{}", jump, when_false, labels)
    }
}

