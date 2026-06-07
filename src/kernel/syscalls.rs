use crate::print;
use crate::kernel::exceptions::ExceptionContext;

pub fn handle_syscall(ctx: &ExceptionContext) -> () {
    // when it was because of `svc`, 
    // lower 16 bits of the ESR will have the syscall number.
    let syscall_number: u16 = (ctx.esr & 0xffff) as u16; 

    match syscall_number {
        1 => sys_print(ctx).unwrap(),
        _ => {
            print!("Unknown syscall: {}", ctx.x[8]).unwrap();
        }
    }
}

/* SYSCALL #1 -- PRINT */
// expects x0 to have the pointer to the string and x1 to have the length of the string. 
fn sys_print(ctx: &ExceptionContext) -> core::fmt::Result {
    let ptr = ctx.x[0] as *const u8;
    let len = ctx.x[1] as usize;

    let s = unsafe { core::slice::from_raw_parts(ptr, len) };
    let s = core::str::from_utf8(s).unwrap_or("");

    print!("{}", s).unwrap();
    Ok(())
}