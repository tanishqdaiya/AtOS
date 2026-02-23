use core::arch::asm;

pub fn get_current_el() -> u64 {
    let el: u64;
    unsafe {
        asm!("mrs {}, CurrentEL", out(reg) el);
    }
    el
}