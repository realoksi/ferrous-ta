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

pub struct SlidingWindow<T, const N: usize> {
    buf: [T; N],
    idx: usize,
    len: usize,
}

impl<T, const N: usize> SlidingWindow<T, N>
where
    T: Copy,
{
    /// ```
    /// use ferrous_ta::*;
    ///
    /// let sliding_window = SlidingWindow::<_, 12>::new(0.0);
    /// ```
    pub fn new(nil: T) -> Self {
        assert!(N > 0);

        Self {
            buf: [nil; N],
            idx: 0,
            len: 0,
        }
    }

    /// ```
    /// use ferrous_ta::*;
    ///
    /// let mut sliding_window = SlidingWindow::<_, 3>::from_slice(&[3, 6, 12, 24]).unwrap();
    /// ```
    #[inline]
    pub fn from_slice(slice: &[T]) -> Option<Self> {
        assert!(N > 0);

        let buf = slice.last_chunk::<N>()?;

        Some(Self {
            buf: *buf,
            idx: 0, // `idx` should always be aligned when initialized this way
            len: N,
        })
    }

    /// ```
    /// use ferrous_ta::*;
    ///
    /// let mut sliding_window = SlidingWindow::<_, 2>::new(0.0);
    ///
    /// assert_eq!(None, sliding_window.push(16.5));
    /// assert_eq!(None, sliding_window.push(20.0));
    /// // Now the window is full. So the next push will result in an eviction of the oldest value.
    /// assert_eq!(Some(16.5), sliding_window.push(21.0));
    /// ```
    #[inline]
    pub fn push(&mut self, value: T) -> Option<T> {
        let prev = self.buf[self.idx];

        self.buf[self.idx] = value;
        self.idx = (self.idx + 1) % N;

        if self.len < N {
            self.len += 1;
            None
        } else {
            Some(prev)
        }
    }

    /// ```
    /// use ferrous_ta::*;
    ///
    /// let mut sliding_window = SlidingWindow::<_, 2>::new(0.0);
    /// assert_eq!(Some(12.7), sliding_window.push_many(&[12.7, 13.6, 14.5]));
    /// ```
    #[inline]
    pub fn push_many(&mut self, values: &[T]) -> Option<T> {
        let mut last = None;

        for &value in values {
            last = self.push(value);
        }

        last
    }

    /// ```
    /// use ferrous_ta::*;
    ///
    /// const PERIODS: usize = 25;
    ///
    /// let mut sliding_window = SlidingWindow::<_, PERIODS>::new(0);
    /// assert_eq!(sliding_window.capacity(), PERIODS);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }

    /// ```
    /// use ferrous_ta::*;
    ///
    /// let mut sliding_window = SlidingWindow::<_, 4>::new(0);
    ///
    /// sliding_window.push_many(&[1, 2, 3]);
    ///
    /// assert_eq!(sliding_window.length(), 3);
    /// ```
    #[inline]
    pub fn length(&self) -> usize {
        self.len
    }

    /// ```
    /// use ferrous_ta::*;
    ///
    /// let mut sliding_window = SlidingWindow::<_, 2>::new(0);
    ///
    /// sliding_window.push_many(&[31, 33]);
    ///
    /// assert_eq!(sliding_window.index(), 0);
    ///
    /// sliding_window.push(28);
    ///
    /// assert_eq!(sliding_window.index(), 1);
    /// ```
    #[inline]
    pub fn index(&self) -> usize {
        self.idx
    }

    /// ```
    /// use ferrous_ta::*;
    ///
    /// let mut sliding_window = SlidingWindow::<_, 3>::from_slice(&[0, 1, 2, 3, 4]).unwrap();
    ///
    /// assert_eq!(*sliding_window.front(), 2);
    /// ```
    #[inline]
    pub fn front(&self) -> &T {
        assert!(self.len > 0);

        if self.len < N {
            &self.buf[0]
        } else {
            &self.buf[self.idx]
        }
    }

    /// ```
    /// use ferrous_ta::*;
    ///
    /// let mut sliding_window = SlidingWindow::<_, 3>::from_slice(&[4, 9, 10]).unwrap();
    ///
    /// assert_eq!(*sliding_window.back(), 10);
    /// ```
    #[inline]
    pub fn back(&self) -> &T {
        assert!(self.len > 0);

        if self.len < N {
            &self.buf[self.len - 1]
        } else {
            &self.buf[(self.idx + N - 1) % N]
        }
    }

    /// ```
    /// use ferrous_ta::*;
    ///
    /// let slice = [90, 91, 97, 99, 101, 104, 105];
    ///
    /// let mut sliding_window = SlidingWindow::<_, 4>::from_slice(&slice).unwrap();
    ///
    /// assert_eq!(*sliding_window.at(1), slice[4]);
    /// ```
    #[inline]
    pub fn at(&self, index: usize) -> &T {
        assert!(self.len > 0);

        let a = index % self.len;

        if self.len < N {
            &self.buf[a]
        } else {
            &self.buf[(self.idx + a) % N]
        }
    }

    /// ```
    /// use ferrous_ta::*;
    ///
    /// let mut sliding_window = SlidingWindow::<_, 3>::from_slice(&[15, 14, 11]).unwrap();
    ///
    /// sliding_window.push_many(&[9, 6]);
    ///
    /// assert_eq!(sliding_window.as_slices(), (&[11][..], &[9, 6][..]));
    /// ```
    #[inline]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        if self.len < N {
            (&self.buf[..self.len], &[])
        } else {
            let (left, right) = self.buf.split_at(self.idx);
            (right, left)
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        // no need to clear `buf` here
        self.idx = 0;
        self.len = 0;
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let (a, b) = self.as_slices();
        a.iter().chain(b.iter())
    }
}

pub struct Accumulator<T, const N: usize> {
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
