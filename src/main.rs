#![no_std]
#![no_main]

use core::fmt::Write;

mod kernel;

use kernel::utils::*;
use kernel::peripherals::*;

fn delay(mut count: u64) {
    while count > 0 {
        unsafe { core::arch::asm!("nop"); }
        count -= 1;
    }
}

#[no_mangle]
pub extern "C" fn _rust_main() -> ! {

    println!("Welcome to, AtOS.").unwrap();

    loop {
        println!("hello from rust {}", get_current_el()).unwrap();
        delay(500_000);
    }
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
