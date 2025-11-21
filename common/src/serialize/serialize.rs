#[inline(always)]
pub fn as_bytes<T>(msg: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts((msg as *const T) as *const u8, size_of::<T>()) }
}

#[inline(always)]
pub fn from_bytes<T>(buf: &[u8]) -> &T {
    unsafe { &*(buf.as_ptr() as *const T) }
}
