#![allow(unused)]

use core::ptr::{read_volatile, write_volatile};

/* 
~~~~ THE DOCUMENTATION FOR THE QA7 COMPONENT CAN BE FOUND AT   
https://github.com/Tekki/raspberrypi-documentation/blob/b1df6ea8e135254e5feb0c8bb036b2a18db8b859/hardware/raspberrypi/bcm2836/QA7_rev3.4.pdf  
*/

// QA7 Base for BCM2837 (Raspberry Pi 3)
const QA7_BASE:             usize    = 0x4000_0000;

// Core timers interrupts
pub const CORE_0_CNT_INT_CTL:   *mut u32 = (QA7_BASE + 0x40) as *mut u32;

// Core interrupt sources
pub const CORE_0_IRQ_SRC:       *const u32 = (QA7_BASE + 0x60) as *const u32;
pub const CORE_0_FIQ_SRC:       *const u32 = (QA7_BASE + 0x70) as *const u32;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum TimerInterruptSource {
    Physical,
    PhysicalNonSecure,
    Hypervisor,
    Virtual,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum InterruptRoute {
    IRQ,
    FIQ,
} 

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum InterruptSource {
    PhysicalSecureTimer = 1 << 0,
    PhysicalNonSecureTimer = 1 << 1,
    HypervisorTimer = 1 << 2,
    VirtualTimer = 1 << 3,
    Mailbox0 = 1 << 4,
    Mailbox1 = 1 << 5,
    Mailbox2 = 1 << 6,
    Mailbox3 = 1 << 7,
    GPU = 1 << 8,
    PMU = 1 << 9,
    AXI = 1 << 10,
    LocalTimer = 1 << 11,
    PeripheralInterrupt = 0x3f << 12, // last 6 bits are for peripheral interrupts
    // \TODO don't know yet how they are implemented and used so for now i just wrote it this way
    // so (irq_sources | PeripheralInterrupt) would give you all the peripheral interrupts (presumably).
}

pub struct Interrupts;

impl Interrupts {
    pub fn daif_unmask_all() {
        unsafe {
            core::arch::asm!("msr DAIFClr, 0b1111", options(nostack, preserves_flags));
        }
    }

    pub fn daif_mask_all() {
        unsafe {
            core::arch::asm!("msr DAIFSet, 0b1111", options(nostack, preserves_flags));
        }
    }
    
    pub fn deroute_timer_interrupt(source: TimerInterruptSource) {
        let mut disable_bit: u32 = match source {
            TimerInterruptSource::Physical => 1 << 0,
            TimerInterruptSource::PhysicalNonSecure => 1 << 1,
            TimerInterruptSource::Hypervisor => 1 << 2,
            TimerInterruptSource::Virtual => 1 << 3,
        };

        // disabling the FIQ as well
        disable_bit = disable_bit | (disable_bit << 4);

        unsafe {
            let current_val: u32 = read_volatile(CORE_0_CNT_INT_CTL);
            write_volatile(CORE_0_CNT_INT_CTL, current_val & !disable_bit);
        }
    }

    pub fn route_timer_interrupt(source: TimerInterruptSource, route: InterruptRoute) {
        // first clearing the interrupt routes for the given source
        Self::deroute_timer_interrupt(source);

        let mut enable_bit: u32 = match source {
            TimerInterruptSource::Physical => 1 << 0,
            TimerInterruptSource::PhysicalNonSecure => 1 << 1,
            TimerInterruptSource::Hypervisor => 1 << 2,
            TimerInterruptSource::Virtual => 1 << 3,
        };
        
        enable_bit = match route {
            InterruptRoute::IRQ => enable_bit,
            InterruptRoute::FIQ => enable_bit << 4,
        };

        unsafe {
            let current_val: u32 = read_volatile(CORE_0_CNT_INT_CTL);
            write_volatile(CORE_0_CNT_INT_CTL, current_val | enable_bit);
        }
    }

    pub fn pending_irq() -> u32 {
        unsafe { read_volatile(CORE_0_IRQ_SRC) }
    }

    pub fn pending_fiq() -> u32 {
        unsafe { read_volatile(CORE_0_FIQ_SRC) }
    }

    pub fn is_irq_pending(source: InterruptSource) -> bool {
        (Self::pending_irq() & (source as u32)) != 0
    }

    pub fn is_fiq_pending(source: InterruptSource) -> bool {
        (Self::pending_fiq() & (source as u32)) != 0
    }
}