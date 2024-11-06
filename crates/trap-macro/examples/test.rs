use trap_marco::kernel_trap;

struct A;

#[kernel_trap(context = A)]
#[link_section = ".bss.stack"]
pub fn handler() {
    println!("1");
}

fn main() {
    unsafe { handler() };
}
