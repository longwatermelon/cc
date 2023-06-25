use crate::error::Error;
use crate::node::{Node, NodeVariant};

pub struct Gen {
    data: String,
    strnum: i32
}

impl Gen {
    pub fn new() -> Self {
        Self { data: String::new(), strnum: 0 }
    }

    pub fn gen(&mut self, root: &Node) -> Result<String, Error> {
        self.data = String::from("section .rodata\n");
        let start: String = String::from("global _start\nsection .text\n_start:\ncall main\nmov rdi, rax\nmov rax, 60\nsyscall\n\n");
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
            _ => todo!()
        }
    }

    fn gen_cpd(&mut self, n: &Node) -> Result<String, Error> {
        let NodeVariant::Cpd { values } = n.variant.as_ref() else { panic!() };
        let mut res: String = String::new();

        for n in values {
            res.push_str(self.gen_expr(n)?.as_str());
        }

        Ok(res)
    }

    fn gen_fdef(&mut self, n: &Node) -> Result<String, Error> {
        let NodeVariant::Fdef { name, params: _, body, rtype: _ } = n.variant.as_ref() else { panic!() };
        let res: String = format!("{}:\npush rbp\nmov rbp, rsp\n{}\nmov rsp, rbp\npop rbp\nret\n\n", name, self.gen_expr(body)?);

        Ok(res)
    }

    fn gen_return(&mut self, n: &Node) -> Result<String, Error> {
        let NodeVariant::Return { value } = n.variant.as_ref() else { panic!() };
        Ok(format!("mov rax, {}", self.gen_expr(value)?))
    }

    fn gen_str(&mut self, value: String) -> Result<String, Error> {
        self.data.push_str(
            format!(
                "str{}: db \"{}\", 10\nstr{}len: equ $ - str{}\n",
                self.strnum, value, self.strnum, self.strnum
            ).as_str()
        );
        self.strnum += 1;

        Ok(String::new())
    }
}

