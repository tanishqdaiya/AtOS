#![allow(dead_code)]

use crate::println;
use crate::kernel::interrupts::{Interrupts, TimerInterruptSource, InterruptRoute};


pub struct PhysicalTimer;

impl PhysicalTimer {
    pub fn init_irq(){
        Interrupts::route_timer_interrupt(TimerInterruptSource::PhysicalNonSecure, InterruptRoute::IRQ);
    }

    // \TODO Physical timers IRQ could be triggered as secure or non secure. 
    // later you may want to add a argument here which lets you differentiate between
    // the two. but for now we only use non secure world so it's fine to just 
    // handle it. 
    pub fn handle_irq() { 
        println!("[P_TIMER] Physical Timer Interrupt Fired!").unwrap();

        // clearing the interrupt by pushing the timer deadline into the future.
        Self::set_cval(Self::read_cnt() + Self::read_frq());
        Self::disable(); // disable the timer so the interrupt doesn't go off again and again.
        // \TODO when implementing "scheduling", then instead of disabling the timer,
        // you may instead want to just set it to the next deadline when cup should 
        // be preempted. but right now with no scheduling implemented, 
        // im just disabling the timer since no code has a standard use for it right now.
    }


    #[inline(always)]
    pub fn read_cnt() -> u64 {
        let value: u64;
        unsafe {
            core::arch::asm!(
                "mrs {val}, CNTPCT_EL0",
                val = out(reg) value,
                options(nostack, preserves_flags)
            );
        }
        value
    }

    #[inline(always)]
    pub fn read_frq() -> u64 {
        let value: u64;
        unsafe {
            core::arch::asm!(
                "mrs {val}, CNTFRQ_EL0",
                val = out(reg) value,
                options(nostack, preserves_flags)
            );
        }
        value
    }

    #[inline(always)]
    pub fn read_ctl() -> u64 {
        let value: u64;
        unsafe {
            core::arch::asm!(
                "mrs {val}, CNTP_CTL_EL0",
                "isb",
                val = out(reg) value,
                options(nostack, preserves_flags)
            );
        }
        value
    }

    #[inline(always)]
    fn write_ctl(val: u64) {
        unsafe {
            core::arch::asm!(
                "msr CNTP_CTL_EL0, {val}",
                "isb",
                val = in(reg) val,
                options(nostack, preserves_flags)
            );
        }
    }

    #[inline(always)]
    pub fn enable() {
        let mut ctl = Self::read_ctl();
        ctl |= 1 << 0; 
        Self::write_ctl(ctl);
    }

    #[inline(always)]
    pub fn disable() {
        let mut ctl = Self::read_ctl();
        ctl &= !1; 
        Self::write_ctl(ctl);
    }

    #[inline(always)]
    pub fn mask_int() {
        let mut ctl = Self::read_ctl();
        ctl |= 1 << 1;
        Self::write_ctl(ctl);
    }

    #[inline(always)]
    pub fn unmask_int() {
        let mut ctl = Self::read_ctl();
        ctl &= !(1 << 1);
        Self::write_ctl(ctl);
    }

    #[inline(always)]
    pub fn set_cval(new_cval: u64) {
        unsafe {
            core::arch::asm!(
                "msr CNTP_CVAL_EL0, {val}",
                "isb",
                val = in(reg) new_cval,
            );
        }
    }

    #[inline(always)]
    pub fn set_tval(new_tval: i32) {
        unsafe {
            core::arch::asm!(
                "msr CNTP_TVAL_EL0, {val:w}",
                "isb",
                val = in(reg) new_tval,
            );
        }
    }

    #[inline(always)]
    pub fn start() {
        Self::unmask_int();
        Self::enable();
    }

    pub fn set_seconds(seconds: u64) {
        let freq = Self::read_frq();
        let now = Self::read_cnt();

        let ticks = freq
            .checked_mul(seconds)
            .expect("Physical Timer Seconds Overflow.");

        Self::set_cval(now + ticks);
        println!("[TIMER] p cval was set.. {} seconds", seconds).unwrap();
    }
}