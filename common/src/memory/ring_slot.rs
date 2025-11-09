use crate::memory::memory::item;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct RingSlot<T> {
    pub(crate) seq: AtomicU32,
    pub(crate) msg: UnsafeCell<T>,
}

impl<T> RingSlot<T> {
    pub fn new() -> Self {
        RingSlot {
            seq: AtomicU32::new(0),
            msg: UnsafeCell::new(item::<T>()),
        }
    }

    pub fn store(&self, seq: u32, val: T) {
        unsafe {
            *self.msg.get() = val;
        }

        self.seq.store(seq, Ordering::Release);
    }

    pub fn load(&self, expected_seq: u32) -> Option<&T> {
        let seq_at_slot = self.seq.load(Ordering::Acquire);

        if seq_at_slot != expected_seq {
            return None;
        }
        unsafe { Some(&*self.msg.get()) }
    }
}

unsafe impl<T> Sync for RingSlot<T> {}
