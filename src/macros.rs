#![macro_use]

/// Get offset to struct member, similar to `offset_of` in C/C++
/// From [here](https://stackoverflow.com/questions/40310483/how-to-get-pointer-offset-in-bytes/40310851#40310851)
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(ptr::null() as *const $ty)).$field as *const _ as usize
    };
}
