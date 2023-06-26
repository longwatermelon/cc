use crate::error::Error;
use crate::node::{Node, NodeVariant, Dtype};
use crate::scope::Scope;

pub struct Gen {
    scope: Scope,
    data: String,
    strnum: i32
}

impl Gen {
    pub fn new() -> Self {
        Self {
            scope: Scope::new(),
            data: String::new(),
            strnum: 0
        }
    }

    pub fn gen(&mut self, root: &Node) -> Result<String, Error> {
        self.data = String::from("section .rodata\n");
        let start: String = String::from("global _start\nsection .text\n_start:\n\tcall main\n\tmov rdi, rax\n\tmov rax, 60\n\tsyscall\n");
        let body: String = self.gen_expr(root)?;

        Ok(format!("{}{}\n{}", start, body, self.data))
    }

    fn gen_expr(&mut self, n: &Node) -> Result<String, Error> {
        match n.variant.as_ref() {
            NodeVariant::Cpd {..} => self.gen_cpd(n),
            NodeVariant::Fdef {..} => self.gen_fdef(n),
            NodeVariant::Return {..} => self.gen_return(n),
            NodeVariant::Int { value } => Ok(value.to_string()),
            NodeVariant::Str { value } => self.gen_str(value.clone()),
            NodeVariant::Char { value } => Ok((*value as u8).to_string()),
            NodeVariant::Vardef {..} => self.gen_vardef(n),
            NodeVariant::Var {..} => self.gen_var(n),
            _ => panic!("{:?} not implemented yet", n.variant)
        }
    }

    fn gen_cpd(&mut self, n: &Node) -> Result<String, Error> {
        let NodeVariant::Cpd { values } = n.variant.as_ref() else { unreachable!() };
        let mut res: String = String::new();

        for n in values {
            res.push_str(self.gen_expr(n)?.as_str());
        }

        Ok(res)
    }

    fn gen_fdef(&mut self, n: &Node) -> Result<String, Error> {
        self.scope.push_layer();
        let NodeVariant::Fdef { name, body, rtype: _, .. } = n.variant.as_ref() else { unreachable!() };
        self.scope.push_fdef(n);
        self.scope.push_fdef_params(name.clone());
        let res: String = format!("\n{}:\n\tpush rbp\n\tmov rbp, rsp\n{}\n\n\tmov rsp, rbp\n\tpop rbp\n\tret\n", name, self.gen_expr(body)?);
        self.scope.pop_layer();

        Ok(res)
    }

    fn gen_return(&mut self, n: &Node) -> Result<String, Error> {
        let NodeVariant::Return { value } = n.variant.as_ref() else { unreachable!() };
        Ok(format!("\n\tmov {}, {}", value.dtype(&self.scope).variant.register("ax"), self.gen_expr(value)?))
    }

    fn gen_vardef(&mut self, n: &Node) -> Result<String, Error> {
        self.scope.push_vardef(n);
        let stack_offset: i32 = self.scope.find_vardef(n.vardef_name()).unwrap().stack_offset;
        Ok(format!(
            "\n\tmov {} [rbp{:+}], {}",
            n.vardef_dtype().variant.deref(),
            stack_offset,
            self.gen_expr(&n.vardef_value())?
        ))
    }

    fn gen_var(&mut self, n: &Node) -> Result<String, Error> {
        let offset: i32 = self.scope.find_vardef(n.var_name()).unwrap().stack_offset;
        let dtype: Dtype = n.dtype(&self.scope);
        Ok(format!("{} [rbp{:+}]", dtype.variant.deref(), offset))
    }

    fn gen_str(&mut self, value: String) -> Result<String, Error> {
        self.data.push_str(
            format!(
                "\tstr{}: db \"{}\", 10\n\tstr{}len: equ $ - str{}\n",
                self.strnum, value, self.strnum, self.strnum
            ).as_str()
        );
        self.strnum += 1;

        Ok(String::new())
    }
}

