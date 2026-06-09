#![no_std]
#![no_main]

use user::{entry, println};

fn main() {
    println!("hello this code is running in the init program!").unwrap();

    let mut x = 1;
    println!("x = {}", x).unwrap();
    x += 1;
    println!("x = {}", x).unwrap();
    
    println!("init program is done working, it will now loop forever.").unwrap();
    loop {println!("This is init looping forever!").unwrap(); core::hint::spin_loop();}
}


entry!(main);