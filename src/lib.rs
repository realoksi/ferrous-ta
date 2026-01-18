#![no_std]

use core::ops::{Add, Div, Mul, Sub};
use num_traits::FromPrimitive;

/// Base trait for stateful moving average implementations.
pub trait MovingAverage<T> {
    /// Resets internal state to its initial values.
    fn reset(&mut self);
}

/// Extension trait for moving averages with **conditional** output.
pub trait OptionalMovingAverage<T>: MovingAverage<T> {
    /// Returns `None` until sufficient state has been accumulated.
    fn push_opt(&mut self, value: T) -> Option<T>;
}

/// Extension trait for moving averages with **unconditional** output.
pub trait GuaranteedMovingAverage<T>: MovingAverage<T> {
    /// Always returns a value for each input.
    fn push(&mut self, value: T) -> T;
}

pub trait Number:
Copy
+ Default
+ Add<Output=Self>
+ Sub<Output=Self>
+ Mul<Output=Self>
+ Div<Output=Self>
+ FromPrimitive
{}

impl<T> Number for T
where
    T: Copy
    + Default
    + Add<Output=Self>
    + Sub<Output=Self>
    + Mul<Output=Self>
    + Div<Output=Self>
    + FromPrimitive
{}

pub struct CMA<T> {
    pub(crate) count: usize,
    pub(crate) avg: T,
}

impl<T: Number> CMA<T> {
    pub fn new() -> Self {
        Self {
            count: 0,
            avg: T::default(),
        }
    }
}

impl<T: Number> MovingAverage<T> for CMA<T> {
    #[inline]
    fn reset(&mut self) {
        self.count = 0;
        self.avg = T::default();
    }
}

impl<T: Number> GuaranteedMovingAverage<T> for CMA<T> {
    #[inline]
    fn push(&mut self, value: T) -> T {
        self.count = self.count + 1;

        let diff = value - self.avg;
        self.avg = self.avg + (diff / T::from_usize(self.count).unwrap());

        self.avg
    }
}

pub struct DEMA<T> {
    pub(crate) ema_1: EMA<T>,
    pub(crate) ema_2: EMA<T>,
    pub(crate) two_as_t: T,
}

impl<T: Number> DEMA<T> {
    pub fn new(periods: usize) -> Self {
        Self {
            ema_1: EMA::<T>::new(periods, None, None),
            ema_2: EMA::<T>::new(periods, None, None),
            two_as_t: T::from_i32(2).unwrap(),
        }
    }
}

impl<T: Number> MovingAverage<T> for DEMA<T> {
    #[inline]
    fn reset(&mut self) {
        self.ema_1.reset();
        self.ema_2.reset();
    }
}

impl<T: Number> GuaranteedMovingAverage<T> for DEMA<T> {
    #[inline]
    fn push(&mut self, value: T) -> T {
        let e1 = self.ema_1.push(value);
        let e2 = self.ema_2.push(e1);

        self.two_as_t * e1 - e2
    }
}

pub struct EMA<T> {
    pub(crate) alpha: T,
    pub(crate) beta: T,
    pub(crate) last: Option<T>,
}

impl<T: Number> EMA<T> {
    pub fn new(periods: usize, first: Option<T>, smoothing_constant: Option<usize>) -> Self {
        assert!(periods > 0);

        let alpha = T::from_usize(smoothing_constant.unwrap_or(2)).unwrap()
            / T::from_usize(periods + 1).unwrap();

        Self {
            alpha,
            beta: T::from_i32(1).unwrap() - alpha,
            last: first,
        }
    }
}

impl<T: Number> MovingAverage<T> for EMA<T> {
    #[inline]
    fn reset(&mut self) {
        self.last = None;
    }
}

impl<T: Number> GuaranteedMovingAverage<T> for EMA<T> {
    #[inline]
    fn push(&mut self, value: T) -> T {
        if let Some(last) = self.last {
            let next = self.alpha * value + self.beta * last;

            self.last = Some(next);
            next
        } else {
            self.last = Some(value);

            value
        }
    }
}

pub struct SMA<T, const N: usize> {
    pub(crate) buf: [T; N],
    pub(crate) count: usize,
    pub(crate) divisor: T,
    pub(crate) index: usize,
    pub(crate) rolling_sum: T,
}

impl<T: Number, const N: usize> SMA<T, N> {
    pub fn new() -> Self {
        assert!(N > 0);

        Self {
            buf: [T::default(); N],
            index: 0,
            divisor: T::from_usize(1).unwrap() / T::from_usize(N).unwrap(),
            rolling_sum: T::default(),
            count: 0,
        }
    }
}

impl<T: Number, const N: usize> MovingAverage<T> for SMA<T, N> {
    #[inline]
    fn reset(&mut self) {
        self.buf = [T::default(); N];
        self.index = 0;
        self.rolling_sum = T::default();
        self.count = 0;
    }
}

impl<T: Number, const N: usize> OptionalMovingAverage<T> for SMA<T, N> {
    #[inline]
    fn push_opt(&mut self, value: T) -> Option<T> {
        if self.count < N {
            self.count += 1;
        }

        let last_value = self.buf[self.index];

        self.buf[self.index] = value;
        self.rolling_sum = self.rolling_sum - last_value + value;

        self.index = (self.index + 1) % N;

        if self.count < N {
            None
        } else {
            Some(self.rolling_sum * self.divisor)
        }
    }
}

pub struct TEMA<T> {
    pub(crate) ema_1: EMA<T>,
    pub(crate) ema_2: EMA<T>,
    pub(crate) ema_3: EMA<T>,
    pub(crate) three_as_t: T,
}

impl<T: Number> TEMA<T> {
    pub fn new(periods: usize) -> Self {
        Self {
            ema_1: EMA::<T>::new(periods, None, None),
            ema_2: EMA::<T>::new(periods, None, None),
            ema_3: EMA::<T>::new(periods, None, None),
            three_as_t: T::from_i32(3).unwrap(),
        }
    }
}

impl<T: Number> MovingAverage<T> for TEMA<T> {
    #[inline]
    fn reset(&mut self) {
        self.ema_1.reset();
        self.ema_2.reset();
        self.ema_3.reset();
    }
}

impl<T: Number> GuaranteedMovingAverage<T> for TEMA<T> {
    #[inline]
    fn push(&mut self, value: T) -> T {
        let e1 = self.ema_1.push(value);
        let e2 = self.ema_2.push(e1);
        let e3 = self.ema_3.push(e2);

        self.three_as_t * e1 - self.three_as_t * e2 + e3
    }
}
