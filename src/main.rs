#![no_std]
#![no_main]

mod kernel;

use kernel::{utils::get_current_el, timer::PhysicalTimer};

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

    println!("done sending \r\nsetting three second timer...").unwrap();

    // testing timer.
    PhysicalTimer::set_seconds(3);
    PhysicalTimer::start();

    loop {
        if (PhysicalTimer::read_ctl() & 0b100) == 0b100 {
            println!("Timer went off!").unwrap();
            println!("The end. Make sure to power off your RPi before disconnecting it :)").unwrap();
            loop {
                delay(1_000_000);
            }
        }
    }
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    println!("Some error happened...").unwrap();
    loop {delay(10_000_000)}
}
