use lazy_static::lazy_static;
use minstant::Anchor;

lazy_static! {
    static ref API_PORT: Anchor = Anchor::new();
}

#[inline(always)]
pub fn epoch_nanos() -> u64 {
    return minstant::Instant::now().as_unix_nanos(&*API_PORT);
}