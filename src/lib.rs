#![no_std]
#![allow(clippy::new_without_default)]

pub mod traits;

#[doc(inline)]
pub use crate::traits::*;

use core::ops::{Add, Sub};

struct SlidingWindow<T, const N: usize> {
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

struct Accumulator<T, const N: usize> {
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

/// # Welles Wilder smoothing
pub struct WWS<T> {
    prev: Option<T>,
    periods: T,
}

impl<T> WWS<T>
where
    T: Scalar,
{
    pub fn new(prev: Option<T>, periods: T) -> Self {
        Self { prev, periods }
    }
}

impl<T> Filter for WWS<T>
where
    T: Scalar,
{
    type Input = T;
    type Output = T;

    #[inline]
    fn step(&mut self, value: Self::Input) -> Self::Output {
        let next = match self.prev {
            Some(prev) => prev + ((value - prev) / self.periods),
            None => value,
        };

        self.prev = Some(next);

        next
    }

    #[inline]
    fn reset(&mut self) {
        self.prev = None;
    }
}

/// Exponential moving average
pub struct EMA<T> {
    prev: Option<T>,
    alpha: T,
    alpha_neg: T,
}

impl<T> EMA<T>
where
    T: Scalar,
{
    pub fn new(prev: Option<T>, periods: i32, smoothing_constant: i32) -> Self {
        let one_t = T::from(1);

        let alpha = T::from(smoothing_constant) / (T::from(periods) + one_t);

        Self {
            prev,
            alpha,
            alpha_neg: one_t - alpha,
        }
    }
}

impl<T> Filter for EMA<T>
where
    T: Scalar,
{
    type Input = T;
    type Output = T;

    #[inline]
    fn step(&mut self, value: Self::Input) -> Self::Output {
        let next = match self.prev {
            Some(prev) => self.alpha * value + self.alpha_neg * prev,
            None => value,
        };

        self.prev = Some(next);

        next
    }

    #[inline]
    fn reset(&mut self) {
        self.prev = None;
    }
}

/// # Cumulative moving average
pub struct CMA<T> {
    count: T,
    avg: T,
    zero_t: T,
    one_t: T,
}

impl<T> CMA<T>
where
    T: Scalar,
{
    pub fn new() -> Self {
        let zero_t = T::from(0);

        Self {
            count: zero_t,
            avg: zero_t,
            zero_t,
            one_t: T::from(1),
        }
    }
}

impl<T> Filter for CMA<T>
where
    T: Scalar,
{
    type Input = T;
    type Output = T;

    #[inline]
    fn step(&mut self, value: Self::Input) -> Self::Output {
        self.count = self.count + self.one_t;

        let diff = value - self.avg;
        self.avg = self.avg + (diff / self.count);

        self.avg
    }

    #[inline]
    fn reset(&mut self) {
        self.count = self.zero_t;
        self.avg = self.zero_t;
    }
}

/// # Double exponential moving average
pub struct DEMA<T> {
    ema_1: EMA<T>,
    ema_2: EMA<T>,
    two_t: T,
}

impl<T> DEMA<T>
where
    T: Scalar,
{
    pub fn new(prev: Option<T>, periods: i32, smoothing_constant: i32) -> Self {
        Self {
            ema_1: EMA::new(prev, periods, smoothing_constant),
            ema_2: EMA::new(None, periods, smoothing_constant),
            two_t: T::from(2),
        }
    }
}

impl<T> Filter for DEMA<T>
where
    T: Scalar,
{
    type Input = T;
    type Output = T;

    #[inline]
    fn step(&mut self, value: Self::Input) -> Self::Output {
        let e1 = self.ema_1.step(value);
        let e2 = self.ema_2.step(e1);

        self.two_t * e1 - e2
    }

    #[inline]
    fn reset(&mut self) {
        self.ema_1.reset();
        self.ema_2.reset();
    }
}

/// # Triple exponential moving average
pub struct TEMA<T> {
    ema_1: EMA<T>,
    ema_2: EMA<T>,
    ema_3: EMA<T>,
    three_t: T,
}

impl<T> TEMA<T>
where
    T: Scalar,
{
    pub fn new(prev: Option<T>, periods: i32, smoothing_constant: i32) -> Self {
        Self {
            ema_1: EMA::new(prev, periods, smoothing_constant),
            ema_2: EMA::new(None, periods, smoothing_constant),
            ema_3: EMA::new(None, periods, smoothing_constant),
            three_t: T::from(3),
        }
    }
}

impl<T> Filter for TEMA<T>
where
    T: Scalar,
{
    type Input = T;
    type Output = T;

    #[inline]
    fn step(&mut self, value: Self::Input) -> Self::Output {
        let e1 = self.ema_1.step(value);
        let e2 = self.ema_2.step(e1);
        let e3 = self.ema_3.step(e2);

        self.three_t * e1 - self.three_t * e2 + e3
    }
    #[inline]
    fn reset(&mut self) {
        self.ema_1.reset();
        self.ema_2.reset();
        self.ema_3.reset();
    }
}

/// Simple moving average
pub struct SMA<T, const N: usize> {
    accumulator: Accumulator<T, N>,
    divisor: T,
}

impl<T, const N: usize> SMA<T, N>
where
    T: Scalar,
{
    pub fn new() -> Self {
        assert!(N <= i32::MAX as usize);

        Self {
            accumulator: Accumulator::new(T::from(0)),
            divisor: T::from(1) / T::from(N as i32),
        }
    }
}

impl<T, const N: usize> Filter for SMA<T, N>
where
    T: Scalar,
{
    type Input = T;
    type Output = Option<T>;

    #[inline]
    fn step(&mut self, value: Self::Input) -> Self::Output {
        if let Some(sum) = self.accumulator.push(value) {
            Some(sum * self.divisor)
        } else {
            None
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.accumulator.reset();
    }
}

/// # Weighted moving average
pub struct WMA<T, const N: usize> {
    sliding_window: SlidingWindow<T, N>,
    divisor_neg: T,
    n_t: T,
    rolling_sum: T,
    rolling_weighted_sum: T,
    zero_t: T,
}

impl<T, const N: usize> WMA<T, N>
where
    T: Scalar,
{
    pub fn new() -> Self {
        todo!();
    }
}

impl<T, const N: usize> Filter for WMA<T, N>
where
    T: Scalar,
{
    type Input = T;
    type Output = Option<T>;

    #[inline]
    fn step(&mut self, value: Self::Input) -> Self::Output {
        todo!();
    }

    #[inline]
    fn reset(&mut self) {
        todo!();
    }
}

/// # Volume weighted average price
pub struct VWAP<T, const N: usize> {
    num_buf: Accumulator<T, N>,
    den_buf: Accumulator<T, N>,
}

impl<T, const N: usize> VWAP<T, N>
where
    T: Scalar,
{
    pub fn new() -> Self {
        let zero = T::from(0);

        Self {
            num_buf: Accumulator::new(zero),
            den_buf: Accumulator::new(zero),
        }
    }
}

impl<T, const N: usize> Filter for VWAP<T, N>
where
    T: Scalar,
{
    type Input = [T; 2];
    type Output = Option<T>;

    /// - `value` is a slice containing the current price and the quantity (in that order)
    #[inline]
    fn step(&mut self, value: Self::Input) -> Self::Output {
        let quantity = value[1];
        let price_quantity = value[0] * quantity;

        if let (Some(num_sum), Some(den_sum)) = (
            self.num_buf.push(price_quantity),
            self.den_buf.push(quantity),
        ) {
            Some(num_sum / den_sum)
        } else {
            None
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.num_buf.reset();
        self.den_buf.reset();
    }
}

pub struct HMA<T, const N: usize, const N_HALF: usize, const N_SQRT: usize> {
    wma_1: WMA<T, N>,
    wma_2: WMA<T, N_HALF>,
    wma_3: WMA<T, N_SQRT>,
    two_t: T,
}

impl<T, const N: usize, const N_HALF: usize, const N_SQRT: usize> HMA<T, N, N_HALF, N_SQRT>
where
    T: Scalar,
{
    pub fn new() -> Self {
        Self {
            wma_1: WMA::new(),
            wma_2: WMA::new(),
            wma_3: WMA::new(),
            two_t: T::from(2),
        }
    }
}

impl<T, const N: usize, const N_HALF: usize, const N_SQRT: usize> Filter
    for HMA<T, N, N_HALF, N_SQRT>
where
    T: Scalar,
{
    type Input = T;
    type Output = Option<T>;

    #[inline]
    fn step(&mut self, value: Self::Input) -> Self::Output {
        let w1 = self.wma_1.step(value);
        let w2 = self.wma_2.step(value);

        if let (Some(w1), Some(w2)) = (w1, w2) {
            let raw_hma = (self.two_t * w1) - w2;

            self.wma_3.step(raw_hma)
        } else {
            None
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.wma_1.reset();
        self.wma_2.reset();
        self.wma_3.reset();
    }
}

/// # Kaufman adaptive moving average
pub struct KAMA {}

/// # Zero lag exponential moving average
pub struct ZLEMA {}
