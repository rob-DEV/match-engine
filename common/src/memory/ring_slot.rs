use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

#[repr(align(64))]
pub struct TransportRingSlot<T> {
    pub(crate) seq: AtomicU32,
    pub(crate) msg: UnsafeCell<MaybeUninit<T>>,
    pub(crate) last_nack_ns: AtomicU64,
    pub(crate) pending_nack: AtomicBool,
}

impl<T> TransportRingSlot<T> {
    pub fn new() -> Self {
        TransportRingSlot {
            seq: AtomicU32::new(0),
            msg: UnsafeCell::new(MaybeUninit::uninit()),
            last_nack_ns: AtomicU64::new(0),
            pending_nack: AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn store(&self, seq: u32, val: T) {
        unsafe {
            (*self.msg.get()).write(val);
        }
        self.seq.store(seq, Ordering::Release);
    }

    #[inline]
    pub fn load(&self, expected_seq: u32) -> Option<T> {
        let seq_at_slot = self.seq.load(Ordering::Acquire);

        if seq_at_slot != expected_seq {
            return None;
        }

        let msg = unsafe { (*self.msg.get()).assume_init_read() };

        Some(msg)
    }

    #[inline]
    pub fn set_nack(&self, nack: u64) {
        self.last_nack_ns.store(nack, Ordering::Release);
    }

    #[inline]
    pub fn last_nack(&self) -> u64 {
        self.last_nack_ns.load(Ordering::Acquire)
    }
}

unsafe impl<T> Sync for TransportRingSlot<T> {}
