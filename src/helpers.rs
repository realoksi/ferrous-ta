use core::ops::{Add, Sub};

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

pub(crate) struct SlidingWindow<T, const N: usize> {
    buf: [T; N],
    idx: usize,
    len: usize,
}

impl<T, const N: usize> SlidingWindow<T, N>
where
    T: Copy,
{
    pub fn new(nil: T) -> Self {
        assert!(N > 0);

        Self {
            buf: [nil; N],
            idx: 0,
            len: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) -> Option<T> {
        let prev = self.buf[self.idx];

        self.buf[self.idx] = value;
        self.idx = (self.idx + 1) % N;

        if self.len < N - 1 {
            self.len += 1;
            None
        } else {
            Some(prev)
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub fn front(&self) -> &T {
        &self.buf[(self.idx + 1) % N]
    }

    #[inline]
    pub fn back(&self) -> &T {
        &self.buf[self.idx]
    }

    #[inline]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        if self.len < N - 1 {
            (&self.buf[..self.len], &[])
        } else {
            let (left, right) = self.buf.split_at(self.idx);
            (right, left)
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.idx = 0;
        self.len = 0;
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let (a, b) = self.as_slices();
        a.iter().chain(b.iter())
    }
}

pub(crate) struct Accumulator<T, const N: usize> {
    sliding_window: SlidingWindow<T, N>,
    sum: T,
    nil: T,
}

impl<T, const N: usize> Accumulator<T, N>
where
    T: Copy + Add<Output = T> + Sub<Output = T>,
{
    pub fn new(nil: T) -> Self {
        Self {
            sliding_window: SlidingWindow::new(nil),
            sum: nil,
            nil,
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) -> Option<T> {
        if let Some(prev) = self.sliding_window.push(value) {
            self.sum = self.sum - prev + value;

            Some(self.sum)
        } else {
            self.sum = self.sum + value;

            None
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.sliding_window.reset();
        self.sum = self.nil;
    }
}
