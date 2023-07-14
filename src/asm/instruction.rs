use crate::asm::Gen;

#[cfg(target_arch = "x86_64")]
impl Gen {
    pub fn mov(&mut self, dest: &str, src: &str) -> String {
        format!("\n\tmov {}, {}", dest, src)
    }
}

