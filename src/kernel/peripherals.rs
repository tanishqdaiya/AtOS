use core::{fmt::Write, ptr::{read_volatile, write_volatile}};

const MMIO_BASE: usize = 0x3F000000;
const AUX_BASE: usize = MMIO_BASE + 0x215000;
const AUX_MU_IO: usize = AUX_BASE + 0x40;
const AUX_MU_LSR: usize = AUX_BASE + 0x54;

pub struct Uart;

impl Uart {
    pub fn write_byte(&self, c: u8) {
        unsafe {
            while read_volatile(AUX_MU_LSR as *const u32) & 0x20 == 0 {}
            write_volatile(AUX_MU_IO as *mut u32, c as u32);
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            self.write_byte(b);
        }
        Ok(())
    }
}

pub fn _print(args: core::fmt::Arguments) -> core::fmt::Result {
    let mut uart = Uart;
    uart.write_fmt(args)?;
    Ok(())
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::kernel::peripherals::_print(
            core::format_args!($($arg)*)
        )
    });
}

#[macro_export]
macro_rules! println {
    ($($args:tt)*) => ({
        $crate::kernel::peripherals::_print(
            core::format_args!("{}\r\n", core::format_args!($($args)*))
        )
    });
}