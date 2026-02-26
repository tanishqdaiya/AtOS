use crate::println;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ExceptionType {
    _SYNC,
    _IRQ,
    _FIQ,
    _SE,
}


#[repr(C)]
pub struct ExceptionContext {
    pub etype: ExceptionType, // u8
    pub _padding: [u8; 7], // because this struct follows c style repr
    pub x: [u64; 31],   // x0–x30
    pub elr: u64,
    pub spsr: u64,
    pub esr: u64,
    pub far: u64,
}

// called by `exceptions.s`
#[no_mangle]
pub extern "C" fn handle_exception_el1(ctx: &mut ExceptionContext) {
    println!("An exception has been detected >xD {}", ctx.etype as u8).unwrap();
}