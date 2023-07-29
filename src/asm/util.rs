use super::Gen;
use crate::error::Error;
use crate::node::Node;

pub fn register(reg: char, n: &Node, gen: &Gen) -> Result<String, Error> {
    n.dtype(&gen.scope)?.register(reg, &gen.scope)
}
