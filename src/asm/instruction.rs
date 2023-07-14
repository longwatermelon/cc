use crate::error::Error;
use crate::asm::Gen;
use crate::node::{Node, Dtype};

pub enum MovArg<'a> {
    Node(&'a Node),
    /// Full register name
    Register(&'a str),
    Stack(Dtype, i32),
}

impl<'a> MovArg<'a> {
    fn repr(&self, gen: &mut Gen) -> Result<String, Error> {
        match self {
            MovArg::Node(n) => gen.gen_repr(n),
            MovArg::Register(reg) => Ok(reg.to_string()),
            MovArg::Stack(dtype, offset) => gen.stack_repr(&dtype, *offset),
        }
    }

    fn associated_register(&self, gen: &mut Gen) -> Result<String, Error> {
        match self {
            MovArg::Node(n) => n.dtype(&gen.scope)?.variant.register('b', &gen.scope),
            MovArg::Register(reg) => Ok(reg.to_string()),
            MovArg::Stack(dtype, _) => dtype.variant.register('b', &gen.scope),
        }
    }
}

#[cfg(target_arch = "x86_64")]
impl Gen {
    pub fn mov(&mut self, dest: MovArg, src: MovArg) -> Result<String, Error> {
        // Get dest and src asm reprs
        let dest_repr: String = dest.repr(self)?;
        let src_repr: String = src.repr(self)?;

        // Avoid mem to mem by moving to a register first
        let reg: String = dest.associated_register(self)?;

        let src_to_reg: String = format!("\n\tmov {}, {}", reg, src_repr);
        let reg_to_dest: String = format!("\n\tmov {}, {}", dest_repr, reg);

        Ok(format!("{}{}", src_to_reg, reg_to_dest))
    }

    pub fn extend_stack(&self, nbytes: i32) -> String {
        format!("\n\tsub rsp, {}", nbytes)
    }
}

