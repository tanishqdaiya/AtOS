#![no_std]
#![no_main]

mod kernel;

use kernel::utils::get_current_el;

fn delay(mut count: u64) {
    while count > 0 {
        unsafe { core::arch::asm!("nop"); }
        count -= 1;
    }
}

#[no_mangle]
pub extern "C" fn _rust_main() -> ! {

    println!("\r\nWelcome to, AtOS.").unwrap();
    println!("Current EL is: EL{}", get_current_el()).unwrap();
    
    delay(10_000_000);
    print!("$ ").unwrap();

    println!("Sending svc in...").unwrap();

    for i in 0..3 {
        println!("{}", 2-i).unwrap();
        delay(10_000_000);
    }

    println!("sending").unwrap();

    unsafe { core::arch::asm!("svc #0"); }

    println!("done sending").unwrap();
    
    loop {delay(5_000_000)}
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    println!("Some error happened...").unwrap();
    loop {delay(10_000_000)}
}
