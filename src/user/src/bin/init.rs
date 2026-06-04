#![no_std]
#![no_main]

use user::println;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("hello").unwrap();
    loop {}
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}