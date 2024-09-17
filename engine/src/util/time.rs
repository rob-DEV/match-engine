use lazy_static::lazy_static;
use minstant::Anchor;

lazy_static! {
    static ref ANCHOR: Anchor = Anchor::new();
}

#[inline(always)]
pub fn epoch_nanos() -> u64 {
    minstant::Instant::now().as_unix_nanos(&*ANCHOR)
}