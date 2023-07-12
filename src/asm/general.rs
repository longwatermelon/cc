use super::Gen;
use crate::error::Error;
use crate::node::{Node, NodeVariant, Dtype, DtypeVariant};
use crate::scope::{ScopeLayer, CVardef, CFdef};

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
        Ok(
            self.gen_expr(value)? +
            &format!(
                "\n\tmov {}, {}",
                value.dtype(&self.scope)?.variant.register("ax", &self.scope)?,
                self.gen_repr(value)?
            )
        )
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

        res.push_str(&format!("\n\tcall {}", name));
        Ok(res)
    }

    pub fn gen_init_list(&mut self, n: &Node) -> Result<String, Error> {
        let mut res: String = String::new();
        let NodeVariant::InitList { dtype: _, fields } = n.variant.as_ref() else { unreachable!() };
        for field in fields {
            self.scope.stack_offset_change_n(&field.1, -1)?;
            res.push_str(self.gen_stack_push(&field.1)?.as_str());
        }

        Ok(res)
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
                    format!(
                        "\n\tsub rsp, {}{}",
                        pushed.dtype(&self.scope)?.variant.num_bytes(&self.scope)?,
                        self.gen_stack_modify(pushed, self.scope.stack_offset())?
                    )
                },
            }
        )
    }

    pub fn gen_stack_modify(&mut self, pushed: &Node, target_stack_offset: i32) -> Result<String, Error> {
        let mut res: String = String::new();
        let mut pushed_repr: String = self.gen_repr(pushed)?;
        // Mem - mem ops not allowed in mov
        if pushed_repr.contains('[') && pushed_repr.contains(']') {
            // Move to register first, change pushed_repr to said register
            let reg: String = pushed.dtype(&self.scope)?.variant.register("bx", &self.scope)?;
            res.push_str(&format!("\n\tmov {}, {}", reg, pushed_repr));
            pushed_repr = reg;
        }

        Ok(res + format!(
            "\n\tmov {} [rbp{:+}], {}",
            pushed.dtype(&self.scope)?.variant.deref(&self.scope)?,
            target_stack_offset,
            pushed_repr
        ).as_str())
    }

    pub fn gen_var(&mut self, _n: &Node) -> Result<String, Error> {
        Ok(String::new())
    }

    pub fn gen_str(&mut self, value: String) -> Result<String, Error> {
        self.data.push_str(
            &format!(
                "\tstr{}: db \"{}\", 10\n\tstr{}len: equ $ - str{}\n",
                self.strnum, value, self.strnum, self.strnum
            )
        );
        self.strnum += 1;

        Ok(String::new())
    }
}
