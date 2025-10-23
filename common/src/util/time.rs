use nix::time::{clock_gettime, ClockId};
use std::thread::sleep;
use std::time::Duration;

#[inline(always)]
pub fn system_nanos() -> u64 {
    let ts = clock_gettime(ClockId::CLOCK_REALTIME).unwrap();
    ts.tv_sec() as u64 * 1_000_000_000 + ts.tv_nsec() as u64
}

#[inline(always)]
pub fn wait_50_milli() {
    sleep(Duration::from_millis(50));
}
