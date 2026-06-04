#![no_std]
#![no_main]

use user::println;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("hello this code is running in the init program!").unwrap();

    let mut x = 1;
    println!("x = {}", x).unwrap();
    x += 1;
    println!("x = {}", x).unwrap();
    
    println!("init program is done working, it will now loop forever.").unwrap();
    loop {}
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("Some error occurred in the init program!").unwrap();
    loop {}
}