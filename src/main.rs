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

    unsafe {
        core::arch::asm!("
    // enable IRQ for Physical timer non secure core 0
    ldr x0, =0x40000040
    mov w1, #0x02
    str w1, [x0]
    msr DAIFClr, #0x2")
    }

    println!("\r\nWelcome to, AtOS.").unwrap();
    println!("Current EL is: EL{}", get_current_el()).unwrap();
    
    delay(10_000_000);
    println!("$ TEST 0").unwrap();

    // testing timer.
    PhysicalTimer::set_seconds(5);
    PhysicalTimer::start();

    println!("Sending svc in...").unwrap();

    for i in 0..3 {
        println!("{}", 2-i).unwrap();
        delay(10_000_000);
    }

    println!("sending").unwrap();

    unsafe { core::arch::asm!("svc #0"); }

    println!("done sending \r\nsetting three second timer...").unwrap();

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
fn panic(_panic: &PanicInfo) -> ! {
    println!("Some error happened...").unwrap();
    loop {delay(10_000_000)}
}
