use std::mem::MaybeUninit;

pub fn uninitialized_arr<T, const COUNT: usize>() -> [T; COUNT] {
    let buf: MaybeUninit<[T; COUNT]> = MaybeUninit::<[T; COUNT]>::uninit();
    let buffer = unsafe { buf.assume_init() };
    buffer
}