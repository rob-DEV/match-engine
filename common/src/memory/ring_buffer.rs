use std::sync::atomic::{AtomicU32, Ordering};

pub struct RingBuffer {
    buf: Vec<AtomicU32>,
    head: AtomicU32,
    tail: AtomicU32,
    cap_mask: u32,
}

impl RingBuffer {
    pub fn new(capacity_pow2: usize) -> Self {
        let cap = capacity_pow2.next_power_of_two();
        RingBuffer {
            buf: (0..cap).map(|_| AtomicU32::new(0)).collect(),
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            cap_mask: (cap as u32) - 1,
        }
    }

    #[inline]
    pub fn push(&self, seq: u32) {
        let h = self.head.load(Ordering::Relaxed);

        let next_h = h.wrapping_add(1);
        self.buf[(h & self.cap_mask) as usize].store(seq, Ordering::Relaxed);
        self.head.store(next_h, Ordering::Release);
    }

    #[inline]
    pub fn pop(&self) -> Option<u32> {
        let t = self.tail.load(Ordering::Relaxed);
        let h = self.head.load(Ordering::Acquire);
        if t == h {
            return None;
        }
        let seq = self.buf[(t & self.cap_mask) as usize].load(Ordering::Relaxed);
        self.tail.store(t.wrapping_add(1), Ordering::Release);
        Some(seq)
    }
}
