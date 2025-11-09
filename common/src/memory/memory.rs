use std::mem::MaybeUninit;

#[inline(always)]
pub fn item<T>() -> T {
    let buf: MaybeUninit<T> = MaybeUninit::<T>::uninit();
    let item = unsafe { buf.assume_init() };
    item
}

#[inline(always)]
pub fn uninitialized_arr<T, const COUNT: usize>() -> [T; COUNT] {
    let buf: MaybeUninit<[T; COUNT]> = MaybeUninit::<[T; COUNT]>::uninit();
    let buffer = unsafe { buf.assume_init() };
    buffer
}
