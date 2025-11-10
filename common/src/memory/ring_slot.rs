use crate::memory::memory::item;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

#[repr(align(64))]
pub struct TransportRingSlot<T> {
    pub(crate) seq: AtomicU32,
    pub(crate) msg: UnsafeCell<T>,
    pub(crate) last_nack_ns: AtomicU64,
}

impl<T> TransportRingSlot<T> {
    pub fn new() -> Self {
        TransportRingSlot {
            seq: AtomicU32::new(0),
            msg: UnsafeCell::new(item::<T>()),
            last_nack_ns: AtomicU64::new(0),
        }
    }

    pub fn store(&self, seq: u32, val: T) {
        unsafe {
            *self.msg.get() = val;
        }

        self.seq.store(seq, Ordering::Release);
        self.last_nack_ns.store(0, Ordering::Release);
    }

    pub fn load(&self, expected_seq: u32) -> Option<&T> {
        let seq_at_slot = self.seq.load(Ordering::Acquire);

        if seq_at_slot != expected_seq {
            return None;
        }
        unsafe { Some(&*self.msg.get()) }
    }

    pub fn set_nack(&self, now_ns: u64) {
        self.last_nack_ns.store(now_ns, Ordering::Release);
    }

    pub fn clear_nack(&self) {
        self.last_nack_ns.store(0, Ordering::Release);
    }

    pub fn last_nack(&self) -> u64 {
        self.last_nack_ns.load(Ordering::Acquire)
    }
}

unsafe impl<T> Sync for TransportRingSlot<T> {}
