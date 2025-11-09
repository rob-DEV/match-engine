use lazy_static::lazy_static;
use std::thread::sleep;
use std::time::Duration;
use minstant::Anchor;
// #[inline(always)]
// pub fn system_nanos() -> u64 {
//     let ts = clock_gettime(ClockId::CLOCK_REALTIME).unwrap();
//     ts.tv_sec() as u64 * 1_000_000_000 + ts.tv_nsec() as u64
// }

lazy_static! {
    static ref ANCHOR: Anchor = Anchor::new();
}

#[inline(always)]
pub fn system_nanos() -> u64 {
    minstant::Instant::now().as_unix_nanos(&*ANCHOR)
}

#[inline(always)]
pub fn wait_50_milli() {
    sleep(Duration::from_millis(50));
}
