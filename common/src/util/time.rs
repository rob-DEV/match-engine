use lazy_static::lazy_static;
use minstant::Anchor;
use std::thread::sleep;
use std::time::Duration;

lazy_static! {
    static ref ANCHOR: Anchor = Anchor::new();
}

#[inline(always)]
pub fn epoch_nanos() -> u64 {
    minstant::Instant::now().as_unix_nanos(&*ANCHOR)
}

#[inline(always)]
pub fn wait_50_milli() {
    sleep(Duration::from_millis(50));
}