use super::Gen;
use super::instruction::AsmArg;
use crate::error::Error;
use crate::node::{Node, NodeVariant, Dtype, DtypeVariant};
use crate::scope::ScopeLayer;
use crate::cdefs::{CFdef, CVardef};

impl Gen {
    pub fn gen_cpd(&mut self, n: &Node) -> Result<String, Error> {
        let NodeVariant::Cpd { values } = n.variant.as_ref() else { unreachable!() };
        let mut res: String = String::new();

        for n in values {
            res.push_str(&self.gen_expr(n)?);
        }

        Ok(res)
    }

    pub fn gen_fdef(&mut self, n: &Node) -> Result<String, Error> {
        // Scope switches, so no nesting
        let prev_layer: ScopeLayer = self.scope.pop_layer();

        self.scope.push_layer();
        let NodeVariant::Fdef { name, body, rtype: _, .. } = n.variant.as_ref() else { unreachable!() };

        // Push params into scope so function body can access them
        self.scope.push_fdef(n)?;
        let fdef: CFdef = self.scope.find_fdef(name, n.line)?.clone();
        let NodeVariant::Fdef { params, .. } = fdef.node.variant.as_ref() else { unreachable!() };
        for (i, param) in params.clone().iter().enumerate() {
            self.scope.push_cvardef(&CVardef::new(param, fdef.param_stack_offsets[i]));
        }

        let res: String = if matches!(body.variant.as_ref(), NodeVariant::Noop) {
            String::new()
        } else {
            format!("\n{}:\n\tpush rbp\n\tmov rbp, rsp\n{}\n\n\tmov rsp, rbp\n\tpop rbp\n\tret\n", name, self.gen_expr(body)?)
        };

        self.scope.pop_layer();

        self.scope.push_layer_from(prev_layer);

        Ok(res)
    }

    pub fn gen_return(&mut self, n: &Node) -> Result<String, Error> {
        let NodeVariant::Return { value } = n.variant.as_ref() else { unreachable!() };
        let reg: String = value.dtype(&self.scope)?.variant.register('a', &self.scope)?;
        Ok(format!(
            "{}{}",
            self.mov(AsmArg::Register(reg.as_str()), AsmArg::Node(value))?,
            "\n\tmov rsp, rbp\n\tpop rbp\n\tret\n"
        ))
    }

    pub fn gen_fcall(&mut self, n: &Node) -> Result<String, Error> {
        let mut res: String = String::new();

        // Get args
        let NodeVariant::Fcall { name, args } = n.variant.as_ref() else { unreachable!() };
        let mut passed_args: Vec<Node> = Vec::new();

        // Get params
        let fdef: CFdef = self.scope.find_fdef(name, n.line)?.clone();
        let NodeVariant::Fdef { params, .. } = fdef.node.variant.as_ref() else { unreachable!() };

        // Check if equal
        if args.len() != params.len() {
            return Err(
                Error::new(
                    format!(
                        "function {} takes in {} argument{} but received {}.",
                        name, params.len(),
                        if params.len() == 1 { "" } else { "s" },
                        args.len()
                    ), n.line
                )
            );
        }

        // Fill in argument values to be passed
        for i in 0..args.len() {
            let mut param: Node = params[i].clone();
            let NodeVariant::Vardef { value, dtype: _, .. } = param.variant.as_mut() else { unreachable!() };
            *value = args[i].clone();
            passed_args.push(param);
        }

        // Push in reverse order
        for arg in passed_args.iter().rev() {
            res.push_str(&self.gen_vardef(arg)?);
        }

        // Only the generated assembly is needed, side effect of variables pushed
        // into scope member variable has to be reversed.
        // Pop previously pushed variables off
        for _ in 0..passed_args.len() {
            self.scope.pop_vardef();
        }

        // Same between x86 and x86_64
        res.push_str(&format!("\n\tcall {}", name));
        Ok(res)
    }

    pub fn gen_init_list(&mut self, n: &Node) -> Result<String, Error> {
        let mut res: String = String::new();
        let NodeVariant::InitList { dtype: _, fields } = n.variant.as_ref() else { unreachable!() };
        for field in fields.iter().rev() {
            self.scope.stack_offset_change_n(&field.1, -1)?;
            res.push_str(self.gen_stack_push(&field.1)?.as_str());
        }

        Ok(res)
    }

    pub fn gen_if(&mut self, n: &Node) -> Result<String, Error> {
        let NodeVariant::If { cond, body } = n.variant.as_ref() else { unreachable!() };

        // Evaluate cond
        let zero_node: Node = Node::new(NodeVariant::Int { value: 0 }, n.line);
        let cmp: String = self.cmp(AsmArg::Node(cond), AsmArg::Node(&zero_node))?;

        //     <body>
        // .Lx:
        //     <rest of the program>
        // If cmp is not equal (cond is true) then jump to .Lx
        let label: usize = self.label;
        let body_and_jmp: String = format!(
            "\n\tje .L{}{}\n.L{}:",
            label,
            self.gen_expr(body)?,
            label,
        );
        self.label += 1;

        Ok(format!(
            "{}{}",
            cmp,
            body_and_jmp,
        ))
    }

    pub fn gen_vardef(&mut self, n: &Node) -> Result<String, Error> {
        // First prepare the value before pushing vardef
        // onto stack to prevent holes in the stack.
        let NodeVariant::Vardef { value, .. } = n.variant.as_ref() else { unreachable!() };
        let value_dtype: Dtype = value.dtype(&self.scope)?;
        let n_dtype: Dtype = n.dtype(&self.scope)?;
        if value_dtype != n_dtype {
            return Err(
                Error::new(
                    format!(
                        "attempting to assign value of type '{}' to variable of type '{}'.",
                        value_dtype.variant, n_dtype.variant
                    ), n.line
                )
            );
        }
        let mut res: String = self.gen_expr(value)?;

        self.scope.stack_offset_change_n(n, -1)?;
        self.scope.push_vardef(n, n.line)?;
        res.push_str(&self.gen_stack_push(value)?);

        Ok(res)
    }

    /// Doesn't modify scope stack offset, uses self.scope.stack_offset().
    /// Before you call this function:
    /// * If the stack needs to grow, change the stack offset before this function call.
    /// * gen_expr the value getting pushed onto the stack if needed, this function won't do it.
    pub fn gen_stack_push(&mut self, pushed: &Node) -> Result<String, Error> {
        Ok(
            match pushed.dtype(&self.scope)?.variant {
                // gen_init_list pushes variables onto the stack
                DtypeVariant::Struct {..} => {
                    self.scope.stack_offset_change_n(pushed, 1)?;
                    let pushed: &Node = pushed.strip(&self.scope)?;

                    self.gen_init_list(&pushed.clone())?
                },
                _ => {
                    let nbytes: i32 = pushed.dtype(&self.scope)?
                                            .variant
                                            .num_bytes(&self.scope)?;

                    format!("{}{}",
                        self.extend_stack(nbytes),
                        self.gen_stack_modify(pushed, self.scope.stack_offset())?
                    )
                },
            }
        )
    }

    pub fn gen_stack_modify(&mut self, pushed: &Node, target_stack_offset: i32) -> Result<String, Error> {
        let pushed_dtype: Dtype = pushed.dtype(&self.scope)?;
        self.mov(
            AsmArg::Stack(&pushed_dtype, target_stack_offset),
            AsmArg::Node(pushed)
        )
    }

    pub fn gen_var(&mut self, _n: &Node) -> Result<String, Error> {
        Ok(String::new())
    }
}

