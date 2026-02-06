use core::ops::Sub;

#[inline]
pub(crate) fn partial_max2<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

#[inline]
pub(crate) fn partial_min2<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { b } else { a }
}

#[inline]
pub(crate) fn get_tr<T: Copy + PartialOrd + Sub<Output = T>>(high: T, low: T, prev_close: T) -> T {
    partial_max2(high, prev_close) - partial_min2(low, prev_close)
}
