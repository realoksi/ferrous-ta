#![no_std]
#![allow(clippy::new_without_default)]

pub mod traits;

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
    pub fn new(prev: Option<T>, periods: T, smoothing_constant: i32) -> Self {
        let one_t = T::from(1);

        let alpha = T::from(smoothing_constant) / (periods + one_t);

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
    pub fn new(prev: Option<T>, periods: T, smoothing_constant: i32) -> Self {
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
    pub fn new(prev: Option<T>, periods: T, smoothing_constant: i32) -> Self {
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
    buf: [T; N],
    count: usize,
    divisor_neg: T,
    index: usize,
    rolling_sum: T,
    zero_t: T,
}

impl<T, const N: usize> SMA<T, N>
where
    T: Scalar,
{
    pub fn new() -> Self {
        // `N` must be able to represent `T`, meaning `usize -> T`.
        // 1. Using a `T: From<usize>` constraint solves the issue, but excludes vital numeric types
        // such as the `f64` primitive.
        // 2. `fn new(periods: T)` is inconvenient (the caller shouldn't have to pass the buffer
        // size twice).
        // 3. Casting `N` via `as i32` is fragile (and perhaps misleading), but it's the most
        // practical solution so far. An assertion against `i32::MAX` essentially makes the
        // conversion infallible.

        assert!(N <= i32::MAX as usize);

        let zero_t = T::from(0);

        Self {
            buf: [zero_t; N],
            index: 0,
            divisor_neg: T::from(1) / T::from(N as i32),
            rolling_sum: zero_t,
            count: 0,
            zero_t,
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
            Some(self.rolling_sum * self.divisor_neg)
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.buf = [self.zero_t; N];
        self.index = 0;
        self.rolling_sum = self.zero_t;
        self.count = 0;
    }
}

/// # Weighted moving average
pub struct WMA<T, const N: usize> {
    buf: [T; N],
    count: usize,
    divisor_neg: T,
    n_t: T,
    index: usize,
    rolling_sum: T,
    rolling_weighted_sum: T,
    zero_t: T,
}

impl<T, const N: usize> WMA<T, N>
where
    T: Scalar,
{
    pub fn new() -> Self {
        assert!(N <= i32::MAX as usize);

        let zero_t = T::from(0);
        let one_t = T::from(1);
        let n_t = T::from(N as i32);

        Self {
            buf: [zero_t; N],
            count: 0,
            divisor_neg: one_t / (n_t * (n_t + one_t) / T::from(2)),
            n_t,
            index: 0,
            rolling_sum: zero_t,
            rolling_weighted_sum: zero_t,
            zero_t,
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
        if self.count < N {
            self.count += 1;
        }

        let last_value = self.buf[self.index];

        self.rolling_weighted_sum = self.rolling_weighted_sum - self.rolling_sum + value * self.n_t;

        self.buf[self.index] = value;
        self.rolling_sum = self.rolling_sum - last_value + value;

        self.index = (self.index + 1) % N;

        if self.count < N {
            None
        } else {
            Some(self.rolling_weighted_sum * self.divisor_neg)
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.buf = [self.zero_t; N];
        self.count = 0;
        self.index = 0;
        self.rolling_sum = self.zero_t;
        self.rolling_weighted_sum = self.zero_t;
    }
}

/// # Volume weighted average price
pub struct VWAP<T, const N: usize> {
    buf: [[T; 2]; N],
    rolling_num_sum: T,
    rolling_den_sum: T,
    count: usize,
    index: usize,
    zero_t: T,
}

impl<T, const N: usize> VWAP<T, N>
where
    T: Scalar,
{
    pub fn new() -> Self {
        let zero_t = T::from(0);

        Self {
            buf: [[zero_t; 2]; N],
            rolling_num_sum: zero_t,
            rolling_den_sum: zero_t,
            count: 0,
            index: 0,
            zero_t,
        }
    }
}

impl<T, const N: usize> Filter for VWAP<T, N>
where
    T: Scalar,
{
    /// \[price, quantity\]
    type Input = [T; 2];
    type Output = Option<T>;

    #[inline]
    fn step(&mut self, value: Self::Input) -> Self::Output {
        if self.count < N {
            self.count += 1;
        }

        let q = value[1];
        let pv = value[0] * q;

        let prev_pv = self.buf[self.index][0];
        let prev_q = self.buf[self.index][1];

        self.buf[self.index] = [pv, q];

        self.rolling_num_sum = self.rolling_num_sum - prev_pv + pv;
        self.rolling_den_sum = self.rolling_den_sum - prev_q + q;

        self.index = (self.index + 1) % N;

        if self.count < N {
            None
        } else {
            Some(self.rolling_num_sum / self.rolling_den_sum)
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.buf = [[self.zero_t; 2]; N];
        self.rolling_num_sum = self.zero_t;
        self.rolling_den_sum = self.zero_t;
        self.count = 0;
        self.index = 0;
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
