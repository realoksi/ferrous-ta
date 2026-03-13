#![no_std]
#![allow(clippy::new_without_default)]

pub mod helpers;
pub mod traits;

pub use crate::helpers::*;
#[doc(inline)]
pub use crate::traits::*;

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

/// # Exponential moving average
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

/// # Simple moving average
pub struct SMA<T, const N: usize> {
    acc: Accumulator<T, N>,
    divisor: T,
}

impl<T, const N: usize> SMA<T, N>
where
    T: Scalar,
{
    pub fn new() -> Self {
        assert!(N <= i32::MAX as usize);

        Self {
            acc: Accumulator::new(T::from(0)),
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
        if let Some(sum) = self.acc.push(value) {
            Some(sum * self.divisor)
        } else {
            None
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.acc.reset();
    }
}

/// # Weighted moving average
pub struct WMA<T, const N: usize> {
    sliding_window: SlidingWindow<T, N>,
    divisor: T,
    rolling_sum: T,
    rolling_weighted_sum: T,
    zero: T,
    n: T,
}

impl<T, const N: usize> WMA<T, N>
where
    T: Scalar,
{
    pub fn new() -> Self {
        assert!(N <= i32::MAX as usize);
        let zero = T::from(0);
        let n = T::from(N as i32);

        Self {
            sliding_window: SlidingWindow::new(zero),
            divisor: T::from(2) / (n * (n + T::from(1))),
            rolling_sum: zero,
            rolling_weighted_sum: zero,
            zero,
            n,
        }
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
        if let Some(prev) = self.sliding_window.push(value) {
            self.rolling_weighted_sum =
                self.rolling_weighted_sum - self.rolling_sum + value * self.n;
            self.rolling_sum = self.rolling_sum - prev + value;

            Some(self.rolling_weighted_sum * self.divisor)
        } else {
            self.rolling_sum = self.rolling_sum + value;
            self.rolling_weighted_sum = self.zero;

            for i in 0..self.sliding_window.length() {
                self.rolling_weighted_sum = self.rolling_weighted_sum
                    + *self.sliding_window.at(i) * T::from((i + 1) as i32);
            }

            if self.sliding_window.length() < N {
                None
            } else {
                Some(self.rolling_weighted_sum * self.divisor)
            }
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.sliding_window.reset();
        self.rolling_sum = self.zero;
        self.rolling_weighted_sum = self.zero;
    }
}

/// # Volume weighted average price
pub struct VWAP<T, const N: usize> {
    num_acc: Accumulator<T, N>,
    den_acc: Accumulator<T, N>,
}

impl<T, const N: usize> VWAP<T, N>
where
    T: Scalar,
{
    pub fn new() -> Self {
        let zero = T::from(0);

        Self {
            num_acc: Accumulator::new(zero),
            den_acc: Accumulator::new(zero),
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
            self.num_acc.push(price_quantity),
            self.den_acc.push(quantity),
        ) {
            Some(num_sum / den_sum)
        } else {
            None
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.num_acc.reset();
        self.den_acc.reset();
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
            let raw_hma = (self.two_t * w2) - w1;

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
