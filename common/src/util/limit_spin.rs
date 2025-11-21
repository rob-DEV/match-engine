// #[cfg(feature = "limit_spin")]
pub fn limit_spin() {
    std::thread::sleep(std::time::Duration::from_millis(1));
}

// #[cfg(not(feature = "limit_spin"))]
// pub fn limit_spin() {
//     // no-op
// }
