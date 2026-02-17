#![no_std]
#![no_main]

use core::ptr::{read_volatile, write_volatile};

const MMIO_BASE: usize = 0x3F000000;
const AUX_BASE: usize = MMIO_BASE + 0x215000;
const AUX_MU_IO: usize = AUX_BASE + 0x40;
const AUX_MU_LSR: usize = AUX_BASE + 0x54;

fn uart_write(c: u8) {
    unsafe {
        while read_volatile(AUX_MU_LSR as *const u32) & 0x20 == 0 {}
        write_volatile(AUX_MU_IO as *mut u32, c as u32);
    }
}

fn print(s: &str) {
    for b in s.bytes() {
        uart_write(b);
    }
}

fn delay(mut count: u64) {
    while count > 0 {
        unsafe { core::arch::asm!("nop"); }
        count -= 1;
    }
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    loop {
        print("hello from rust\r\n");
        delay(500_000);
    }
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
