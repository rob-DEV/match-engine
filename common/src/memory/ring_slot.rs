use crate::memory::memory::item;
use std::cell::UnsafeCell;
use std::sync::atomic::AtomicU32;

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
}
unsafe impl<T> Sync for RingSlot<T> {}
