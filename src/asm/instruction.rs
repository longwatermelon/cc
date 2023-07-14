use crate::asm::Gen;

#[cfg(target_arch = "x86_64")]
impl Gen {
    pub fn mov(&self, dest: &str, src: &str) -> String {
        format!("\n\tmov {}, {}", dest, src)
    }

    pub fn extend_stack(&self, nbytes: i32) -> String {
        format!("\n\tsub rsp, {}", nbytes)
    }
}

