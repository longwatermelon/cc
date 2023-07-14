mod general;
mod ops;
mod instruction;

use crate::error::Error;
use crate::scope::Scope;
use crate::cdefs::CVardef;
use crate::node::{Node, NodeVariant, Dtype};
use crate::lexer::TokenType;

pub struct Gen {
    scope: Scope,
    data: String,
}

impl Gen {
    pub fn new() -> Self {
        Self {
            scope: Scope::new(),
            data: String::new(),
        }
    }

    pub fn gen(&mut self, root: &Node) -> Result<String, Error> {
        #[cfg(target_arch = "x86_64")]
        {
            self.data = String::from("section .rodata\n");
            let start: String = String::from("global _start\nsection .text\n_start:\n\tcall main\n\tmov rdi, rax\n\tmov rax, 60\n\tsyscall\n");
            let body: String = self.gen_expr(root)?;

            Ok(format!("{}{}\n{}", start, body, self.data))
        }
    }

    /// Generate instruction(s)
    pub fn gen_expr(&mut self, n: &Node) -> Result<String, Error> {
        match n.variant.as_ref() {
            NodeVariant::Cpd {..} => self.gen_cpd(n),
            NodeVariant::Fdef {..} => self.gen_fdef(n),
            NodeVariant::Return {..} => self.gen_return(n),
            NodeVariant::Vardef {..} => self.gen_vardef(n),
            NodeVariant::Var {..} => self.gen_var(n),
            NodeVariant::Fcall {..} => self.gen_fcall(n),
            NodeVariant::InitList {..} => self.gen_init_list(n),
            NodeVariant::Struct {..} => {
                self.scope.push_struct(n)?;
                Ok(String::new())
            },
            // NodeVariant::Str { value } => self.gen_str(value.clone()),
            NodeVariant::Noop |
            NodeVariant::Str {..} |
            NodeVariant::Int {..} |
            NodeVariant::Char {..} => Ok(String::new()),
            NodeVariant::Binop {..} => self.gen_binop(n),
            _ => panic!("{:?} not implemented yet [EXPR]", n.variant),
        }
    }

    /// Generate an operand
    pub fn gen_repr(&mut self, n: &Node) -> Result<String, Error> {
        match n.variant.as_ref() {
            NodeVariant::Int { value } => Ok(value.to_string()),
            // NodeVariant::Str { value } => self.gen_str(value.clone()),
            NodeVariant::Char { value } => Ok((*value as u8).to_string()),
            NodeVariant::Var { name } => {
                let cv: &CVardef = self.scope.find_vardef(name, n.line)?;
                self.stack_repr(
                    &cv.node.dtype(&self.scope)?,
                    cv.stack_offset
                )
            },
            NodeVariant::Vardef { value, .. } => self.gen_repr(value),
            NodeVariant::Fcall { name, .. } => Ok(self.scope.find_fdef(name, n.line)?.node.dtype(&self.scope)?.variant.register('a', &self.scope)?),
            NodeVariant::Binop { btype: TokenType::Dot, .. } =>
                n.dtype(&self.scope)?.variant.register('b', &self.scope),
            NodeVariant::Binop { btype: TokenType::Plus, .. } |
            NodeVariant::Binop { btype: TokenType::Minus, .. } |
            NodeVariant::Binop { btype: TokenType::Star, .. } |
            NodeVariant::Binop { btype: TokenType::Div, .. } =>
                n.dtype(&self.scope)?.variant.register('a', &self.scope),
            _ => panic!("{:?} not implemented yet [REPR]", n.variant),
        }
    }

    /// Represent stack at some offset as an operand
    pub fn stack_repr(&self, dtype: &Dtype, offset: i32) -> Result<String, Error> {
        #[cfg(target_arch = "x86_64")]
        Ok(format!(
            "{} [rbp{:+}]",
            dtype.variant.deref(&self.scope)?, offset
        ))
    }
}

