// Spinlocks

use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::Cell;
use crate::Interrupts;
use crate::kernel::processes::mycpu;

pub struct Spinlock {
    lock: AtomicBool,
    holding_cpuid: Cell<Option<usize>>,

    // For debugging:
    name: &'static str, // Name of the lock
}

unsafe impl Sync for Spinlock {}

impl Spinlock {
    pub const fn new(name: &'static str) -> Self {
        Self {
            lock: AtomicBool::new(false),
            holding_cpuid: Cell::new(None),
            name,
        }
    }

    pub fn acquire(&self) {
        Interrupts::push_off();

        let current_cpuid = mycpu().cid;

        // Prevent the exact same lock from deadlocking itself.
        // If it is already locked on a single core, we'd spin forever.
        if self.lock.load(Ordering::Relaxed) && self.holding_cpuid.get() == Some(current_cpuid) {
            panic!("acquire({})", self.name);
        }

        while self.lock.compare_exchange(false,
                                         true,
                                         Ordering::Acquire,
                                         Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }

        self.holding_cpuid.set(Some(current_cpuid));
    }

    pub fn release(&self) {
        let current_cpuid = mycpu().cid;
        
        if !self.lock.load(Ordering::Relaxed) {
            panic!("release({})", self.name);
        }

        if self.holding_cpuid.get() != Some(current_cpuid) {
            panic!("Spinlock release error: ({}) owned by CPU {:?}, but CPU {} tried to release it!", 
                   self.name, self.holding_cpuid.get(), current_cpuid);
        }

        self.holding_cpuid.set(None);
        self.lock.store(false, Ordering::Release);

        Interrupts::pop_off();
    }
}
