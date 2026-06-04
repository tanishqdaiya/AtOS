#![no_std]
#![no_main]

mod kernel;

use kernel::{utils::get_current_el, timer::PhysicalTimer};

// this is to read from the linker-- the end of kernel in memory and top of the stack. 
unsafe extern "C" {
    unsafe static _kernel_top: u8;
    unsafe static _stack_top: u8;
}

fn delay(mut count: u64) {
    while count > 0 {
        unsafe { core::arch::asm!("nop"); }
        count -= 1;
    }
}

#[unsafe(no_mangle)]
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
    
    let kernel_end_addr = unsafe { &_kernel_top as *const u8 as usize };
    let stack_top_addr = unsafe { &_stack_top as *const u8 as usize };
    println!("Kernel ends at: {:#x}", kernel_end_addr).unwrap();
    println!("Kernel size: {} KB", kernel_end_addr / 1024).unwrap();
    println!("Stack top at: {:#x}", stack_top_addr).unwrap();

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
