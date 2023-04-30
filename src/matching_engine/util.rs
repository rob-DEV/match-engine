use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_epoch_time() -> u64 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
}

pub fn current_epoch_nano_time() -> u128 {
    let duration_since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    return duration_since_epoch.as_nanos(); // u128
}
