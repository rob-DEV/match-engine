#[inline(always)]
pub fn limit_spin() {
    std::thread::sleep(std::time::Duration::from_micros(10));
}
