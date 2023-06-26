use crate::error::Error;
use crate::node::{Node, NodeVariant};
use crate::scope::{Scope, CVardef, CFdef};

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
            NodeVariant::Vardef {..} => self.gen_vardef(n),
            NodeVariant::Var {..} => self.gen_var(n),
            NodeVariant::Fcall {..} => self.gen_fcall(n),
            NodeVariant::Str { value } => self.gen_str(value.clone()),
            _ => panic!("{:?} not implemented yet [EXPR]", n.variant)
        }
    }

    fn gen_repr(&mut self, n: &Node) -> Result<String, Error> {
        match n.variant.as_ref() {
            NodeVariant::Int { value } => Ok(value.to_string()),
            // NodeVariant::Str { value } => self.gen_str(value.clone()),
            NodeVariant::Char { value } => Ok((*value as u8).to_string()),
            NodeVariant::Var { name } => {
                let cv: &CVardef = self.scope.find_vardef(name.clone()).unwrap();
                Ok(format!("{} [rbp{:+}]", cv.node.dtype(&self.scope).variant.deref(), cv.stack_offset))
            },
            NodeVariant::Vardef { var, .. } => self.gen_repr(var),
            NodeVariant::Fcall { name, .. } => Ok(self.scope.find_fdef(name.clone()).unwrap().node.dtype(&self.scope).variant.register("ax")),
            _ => panic!("{:?} not implemented yet [REPR]", n.variant)
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
        Ok(
            self.gen_expr(value)? +
            format!(
                "\n\tmov {}, {}",
                value.dtype(&self.scope).variant.register("ax"),
                self.gen_repr(value)?
            ).as_str()
        )
    }

    fn gen_fcall(&mut self, n: &Node) -> Result<String, Error> {
        let mut res: String = String::new();

        let NodeVariant::Fcall { name, args } = n.variant.as_ref() else { unreachable!() };
        let mut passed_args: Vec<Node> = Vec::new();

        let fdef: CFdef = self.scope.find_fdef(name.clone()).unwrap().clone();
        let NodeVariant::Fdef { params, .. } = fdef.node.variant.as_ref() else { unreachable!() };

        for i in 0..args.len() {
            let mut param: Node = params[i].clone();
            let NodeVariant::Vardef { value, dtype: _, .. } = param.variant.as_mut() else { unreachable!() };
            *value = args[i].clone();
            passed_args.push(param);
        }

        for arg in &passed_args {
            res.push_str(
                self.gen_vardef(arg)?.as_str()
            );
        }

        res.push_str(format!("\n\tcall {}", name).as_str());

        Ok(res)
    }

    fn gen_vardef(&mut self, n: &Node) -> Result<String, Error> {
        self.scope.push_vardef(n);
        let stack_offset: i32 = self.scope.find_vardef(n.vardef_name()).unwrap().stack_offset;

        let NodeVariant::Vardef { value, .. } = n.variant.as_ref() else { unreachable!() };
        self.gen_stack_push(value, stack_offset)
    }

    fn gen_stack_push(&mut self, pushed: &Node, stack_offset: i32) -> Result<String, Error> {
        let mut res: String = String::new();
        let mut pushed_repr: String = self.gen_repr(pushed)?;
        // Mem - mem ops not allowed in mov
        if pushed_repr.contains("[") && pushed_repr.contains("]") {
            // Move to register first, change pushed_repr to said register
            let reg: String = pushed.dtype(&self.scope).variant.register("bx");
            res.push_str(format!("\n\tmov {}, {}", reg, pushed_repr).as_str());
            pushed_repr = reg;
        }

        res.push_str(
            format!(
                "\n\tsub rsp, {}\n\tmov {} [rbp{:+}], {}",
                pushed.dtype(&self.scope).variant.num_bytes(),
                pushed.dtype(&self.scope).variant.deref(),
                stack_offset,
                pushed_repr
            ).as_str()
        );

        Ok(res)
    }

    fn gen_var(&mut self, _n: &Node) -> Result<String, Error> {
        Ok(String::new())
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

