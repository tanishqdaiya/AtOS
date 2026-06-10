#![no_std]

pub mod stdlib;

#[macro_export]
macro_rules! entry {
    ($main:path) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn _start() -> ! {
            $main();
            crate::stdlib::syscalls::exit(0);
        }

        use core::panic::PanicInfo;
        #[panic_handler]
        fn panic(_info: &PanicInfo) -> ! {
            println!("Rust exception occured in user space!").unwrap();
            crate::stdlib::syscalls::exit(1);
        }
    };
}