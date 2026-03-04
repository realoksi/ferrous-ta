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

/// A generic, fixed-size circular buffer.
///
/// # ConstParams
///
/// - `N`: Maximum amount of elements the sliding window can hold
///
/// # Examples
///
/// ```
/// use ferrous_ta::*;
///
/// let mut sliding_window = SlidingWindow::<_, 3>::new(0);
///
/// sliding_window.push_many(&[41, 24, 80]);
/// ```
pub struct SlidingWindow<T, const N: usize> {
    buf: [T; N],
    idx: usize,
    len: usize,
}

impl<T, const N: usize> SlidingWindow<T, N>
where
    T: Copy,
{
    /// Creates a new instance.
    ///
    /// # Parameters
    ///
    /// - `nil`: An empty value of type `T` to initialize the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use ferrous_ta::*;
    ///
    /// let sliding_window = SlidingWindow::<_, 12>::new(0);
    /// ```
    pub fn new(nil: T) -> Self {
        assert!(N > 0);

        Self {
            buf: [nil; N],
            idx: 0,
            len: 0,
        }
    }

    /// Creates a new instance from a slice.
    ///
    /// # Parameters
    ///
    /// - `slice`: A slice reference to initialize from
    ///
    /// # Returns
    ///
    /// - `Some(Self)`: A new instance was successfully initialized
    /// - `None`: Not enough items in `slice` (`slice.len() < N`)
    ///
    /// # Examples
    ///
    /// Create a new sliding window with a capacity of 3 items, from an existing slice.
    ///
    /// ```
    /// use ferrous_ta::*;
    ///
    /// let slice = [10, 15, 20, 25];
    ///
    /// let sliding_window = SlidingWindow::<_, 3>::from_slice(&slice); // 10 is discarded
    /// ```
    #[inline]
    pub fn from_slice(slice: &[T]) -> Option<Self> {
        assert!(N > 0);

        let buf = slice.last_chunk::<N>()?;

        Some(Self {
            buf: *buf,
            idx: 0, // `idx` should always be aligned (0) when initialized from a slice
            len: N,
        })
    }

    /// Writes a new value to the sliding window.
    ///
    /// When the window is at capacity, the oldest value is ejected and replaced.
    ///
    /// # Parameters
    ///
    /// - `value`: A value to insert into the window
    ///
    /// # Returns
    ///
    /// - `Some(T)`: An ejected value
    /// - `None`: Window hasn't reached its capacity yet
    ///
    /// # Examples
    ///
    /// ```
    /// use ferrous_ta::*;
    ///
    /// let mut sliding_window = SlidingWindow::<_, 2>::new(0.0);
    ///
    /// assert_eq!(None, sliding_window.push(16.5));
    /// assert_eq!(None, sliding_window.push(20.0));
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

    /// Loops over a slice of values and pushes each to the sliding window.
    ///
    /// **Only the last ejected value is returned.**
    ///
    /// # Parameters
    ///
    /// - `values`: A slice of values to insert into the window
    ///
    /// # Returns
    ///
    /// - `Some(T)`: The last ejected value
    /// - `None`: Window hasn't reached its capacity yet
    ///
    /// # Examples
    ///
    /// ```
    /// use ferrous_ta::*;
    ///
    /// let mut sliding_window = SlidingWindow::<_, 2>::new(0.0);
    ///
    /// assert_eq!(Some(16.5), sliding_window.push_many(&[16.5, 20.0, 21.0]));
    /// ```
    #[inline]
    pub fn push_many(&mut self, values: &[T]) -> Option<T> {
        let mut last = None;

        for &value in values {
            last = self.push(value);
        }

        last
    }

    /// Gets the capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }

    /// Gets the current length.
    #[inline]
    pub fn length(&self) -> usize {
        self.len
    }

    /// Gets the current internal index.
    #[inline]
    pub fn index(&self) -> usize {
        self.idx
    }

    /// Gets a reference to the oldest item in the window.
    #[inline]
    pub fn front(&self) -> &T {
        assert!(self.len > 0); // is this assertion necessary?

        if self.len < N {
            &self.buf[0]
        } else {
            &self.buf[self.idx]
        }
    }

    /// Gets a reference to the newest item in the window.
    #[inline]
    pub fn back(&self) -> &T {
        assert!(self.len > 0); // why is this here?

        if self.len < N {
            &self.buf[self.len - 1]
        } else {
            &self.buf[(self.idx + N - 1) % N]
        }
    }

    /// Gets a reference to a value at a specific index of the internal buffer.
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

/// ...
///
/// # ConstParams
///
/// - `N`: Maximum amount of elements the accumulator can hold
///
pub struct Accumulator<T, const N: usize> {
    sliding_window: SlidingWindow<T, N>,
    sum: T,
    nil: T,
}

impl<T, const N: usize> Accumulator<T, N>
where
    T: Copy + Add<Output = T> + Sub<Output = T>,
{
    /// Creates a new instance.
    ///
    /// # Parameters
    ///
    /// - `nil`: A fill value to use when initializing the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use ferrous_ta::*;
    ///
    /// let acc = Accumulator::<_, 5>::new(0);
    /// ```
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
