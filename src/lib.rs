//! # Description
//!
//! A generic `no_std` compatible technical analysis library. Assumes users are familiar with the
//! concepts behind the implemented indicators.

#![no_std]

use core::ops::{Add, Div, Mul, Neg, Sub};
use num_traits::FromPrimitive;

#[inline]
fn partial_max2<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

#[inline]
fn partial_min2<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { b } else { a }
}

/// Base trait for moving average indicators.
pub trait MovingAverage {
    type Input;
    type Output;
    fn push(&mut self, value: Self::Input) -> Self::Output;
    /// Resets internal state to its initial values.
    fn reset(&mut self);
}

/// Base trait for oscillator indicators.
pub trait Oscillator {
    type Input;
    type Output;
    fn push(&mut self, value: Self::Input) -> Self::Output;
    /// Resets internal state to its initial values.
    fn reset(&mut self);
}

/// Base trait for volatility indicators.
pub trait Volatility {
    type Input;
    type Output;
    fn push(&mut self, value: Self::Input) -> Self::Output;
    /// Resets internal state to its initial values.
    fn reset(&mut self);
}

/// Blanket trait for number-like type constraints.
pub trait Number:
    Copy
    + Default
    + PartialOrd
    + Neg<Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + FromPrimitive
{
    // Should we constrain to arbitrary number types, or maybe prefer float-likes?
}

impl<T> Number for T where
    T: Copy
        + Default
        + PartialOrd
        + Neg<Output = Self>
        + Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + FromPrimitive
{
}

/// # Cumulative moving average
///
/// Computes the arithmetic mean of all values ingested so far by incrementally updating an internal
/// accumulator.
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

impl<T: Number> MovingAverage for CMA<T> {
    type Input = T;
    type Output = Option<T>;

    #[inline]
    fn push(&mut self, value: Self::Input) -> Self::Output {
        self.count = self.count + 1;

        let diff = value - self.avg;
        self.avg = self.avg + (diff / T::from_usize(self.count).unwrap());

        Some(self.avg)
    }

    #[inline]
    fn reset(&mut self) {
        self.count = 0;
        self.avg = T::default();
    }
}

/// # Double exponential moving average
pub struct DEMA<T> {
    pub(crate) ema_1: EMA<T>,
    pub(crate) ema_2: EMA<T>,
    pub(crate) two_as_t: T,
}

impl<T: Number> DEMA<T> {
    pub fn new(periods: usize, first: Option<T>, smoothing_constant: Option<usize>) -> Self {
        Self {
            ema_1: EMA::<T>::new(periods, first, smoothing_constant),
            ema_2: EMA::<T>::new(periods, None, smoothing_constant),
            two_as_t: T::from_i32(2).unwrap(),
        }
    }
}

impl<T: Number> MovingAverage for DEMA<T> {
    type Input = T;
    type Output = Option<T>;

    #[inline]
    fn push(&mut self, value: Self::Input) -> Self::Output {
        let e1 = self.ema_1.push(value).unwrap();
        let e2 = self.ema_2.push(e1).unwrap();

        Some(self.two_as_t * e1 - e2)
    }

    #[inline]
    fn reset(&mut self) {
        self.ema_1.reset();
        self.ema_2.reset();
    }
}

/// # Exponential moving average
/// Computes a weighted moving average using exponential decay, emphasizing recent values.
///
/// # Example
/// ```
/// use ferrous_ta::*;
///
/// let mut ema = EMA::new(3, None, None);
///
/// assert_eq!(ema.push(1.0), Some(1.0));
/// assert_eq!(ema.push(2.0), Some(1.5));
/// assert_eq!(ema.push(3.0), Some(2.25));
/// ```
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

impl<T: Number> MovingAverage for EMA<T> {
    type Input = T;
    type Output = Option<T>;

    #[inline]
    fn push(&mut self, value: Self::Input) -> Self::Output {
        if let Some(last) = self.last {
            let next = self.alpha * value + self.beta * last;

            self.last = Some(next);
        } else {
            self.last = Some(value);
        }

        self.last
    }

    #[inline]
    fn reset(&mut self) {
        self.last = None;
    }
}

/// # Simple moving average
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

impl<T: Number, const N: usize> MovingAverage for SMA<T, N> {
    type Input = T;
    type Output = Option<T>;

    #[inline]
    fn push(&mut self, value: Self::Input) -> Self::Output {
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

    #[inline]
    fn reset(&mut self) {
        self.buf = [T::default(); N];
        self.index = 0;
        self.rolling_sum = T::default();
        self.count = 0;
    }
}

/// # Triple exponential moving average
pub struct TEMA<T> {
    pub(crate) ema_1: EMA<T>,
    pub(crate) ema_2: EMA<T>,
    pub(crate) ema_3: EMA<T>,
    pub(crate) three_as_t: T,
}

impl<T: Number> TEMA<T> {
    pub fn new(periods: usize, first: Option<T>, smoothing_constant: Option<usize>) -> Self {
        Self {
            ema_1: EMA::<T>::new(periods, first, smoothing_constant),
            ema_2: EMA::<T>::new(periods, None, smoothing_constant),
            ema_3: EMA::<T>::new(periods, None, smoothing_constant),
            three_as_t: T::from_i32(3).unwrap(),
        }
    }
}

impl<T: Number> MovingAverage for TEMA<T> {
    type Input = T;
    type Output = Option<T>;

    #[inline]
    fn push(&mut self, value: Self::Input) -> Option<T> {
        let e1 = self.ema_1.push(value).unwrap();
        let e2 = self.ema_2.push(e1).unwrap();
        let e3 = self.ema_3.push(e2).unwrap();

        Some(self.three_as_t * e1 - self.three_as_t * e2 + e3)
    }
    #[inline]
    fn reset(&mut self) {
        self.ema_1.reset();
        self.ema_2.reset();
        self.ema_3.reset();
    }
}

/// # Moving average convergence divergence
pub struct MACD<T> {
    pub(crate) fast_ema: EMA<T>,
    pub(crate) slow_ema: EMA<T>,
    pub(crate) signal_ema: EMA<T>,
}

impl<T: Number> MACD<T> {
    pub fn new(
        fast_periods: usize,
        slow_periods: usize,
        signal_periods: usize,
        first: Option<T>,
        smoothing_constant: Option<usize>,
    ) -> Self {
        assert!(fast_periods < slow_periods);

        Self {
            fast_ema: EMA::<T>::new(fast_periods, first, smoothing_constant),
            slow_ema: EMA::<T>::new(slow_periods, None, smoothing_constant),
            signal_ema: EMA::<T>::new(signal_periods, None, smoothing_constant),
        }
    }
}

impl<T: Number> Oscillator for MACD<T> {
    type Input = T;
    type Output = [T; 3];

    #[inline]
    fn push(&mut self, value: Self::Input) -> Self::Output {
        let macd = self.fast_ema.push(value).unwrap() - self.slow_ema.push(value).unwrap();

        let signal = self.signal_ema.push(macd).unwrap();

        [macd, signal, macd - signal]
    }

    #[inline]
    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
    }
}

/// # Weighted moving average
pub struct WMA<T, const N: usize> {
    pub(crate) buf: [T; N],
    pub(crate) count: usize,
    pub(crate) divisor: T,
    pub(crate) n_as_t: T,
    pub(crate) index: usize,
    pub(crate) rolling_sum: T,
    pub(crate) rolling_weighted_sum: T,
}

impl<T: Number, const N: usize> WMA<T, N> {
    pub fn new() -> Self {
        assert!(N > 0);

        Self {
            buf: [T::default(); N],
            count: 0,
            divisor: T::from_usize(1).unwrap() / T::from_usize(N * (N + 1) / 2).unwrap(),
            n_as_t: T::from_usize(N).unwrap(),
            index: 0,
            rolling_sum: T::default(),
            rolling_weighted_sum: T::default(),
        }
    }
}

impl<T: Number, const N: usize> MovingAverage for WMA<T, N> {
    type Input = T;
    type Output = Option<T>;

    #[inline]
    fn push(&mut self, value: Self::Input) -> Self::Output {
        if self.count < N {
            self.count += 1;
        }

        let last_value = self.buf[self.index];

        self.rolling_weighted_sum =
            self.rolling_weighted_sum - self.rolling_sum + value * self.n_as_t;

        self.buf[self.index] = value;
        self.rolling_sum = self.rolling_sum - last_value + value;

        self.index = (self.index + 1) % N;

        if self.count < N {
            None
        } else {
            Some(self.rolling_weighted_sum * self.divisor)
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.buf = [T::default(); N];
        self.count = 0;
        self.index = 0;
        self.rolling_sum = T::default();
        self.rolling_weighted_sum = T::default();
    }
}

/// # Relative strength index
pub struct RSI<T> {
    pub(crate) avg_gain: T,
    pub(crate) avg_loss: T,
    pub(crate) count: usize,
    pub(crate) last: Option<T>,
    pub(crate) one_hundred_as_t: T,
    pub(crate) periods: usize,
    pub(crate) periods_as_t: T,
    pub(crate) periods_minus_one_as_t: T,
    pub(crate) sum_gain: T,
    pub(crate) sum_loss: T,
    pub(crate) zero_as_t: T,
}

impl<T: Number> RSI<T> {
    pub fn new(periods: usize) -> Self {
        assert!(periods > 0);

        Self {
            avg_gain: T::default(),
            avg_loss: T::default(),
            count: 0,
            last: None,
            one_hundred_as_t: T::from_usize(100).unwrap(),
            periods,
            periods_as_t: T::from_usize(periods).unwrap(),
            periods_minus_one_as_t: T::from_usize(periods - 1).unwrap(),
            sum_gain: T::default(),
            sum_loss: T::default(),
            zero_as_t: T::from_usize(0).unwrap(),
        }
    }
}

impl<T: Number> Oscillator for RSI<T> {
    type Input = T;
    type Output = Option<T>;

    #[inline]
    fn push(&mut self, value: Self::Input) -> Self::Output {
        if self.last.is_none() {
            self.last = Some(value);

            return None;
        }

        let last = self.last.unwrap(); // self.last cannot be None
        let diff = value - last;
        let gain = partial_max2(diff, self.zero_as_t);
        let loss = partial_max2(-diff, self.zero_as_t);

        if self.count < self.periods {
            self.sum_gain = self.sum_gain + gain;
            self.sum_loss = self.sum_loss + loss;
            self.count += 1;

            if self.count == self.periods {
                self.avg_gain = self.sum_gain / self.periods_as_t;
                self.avg_loss = self.sum_loss / self.periods_as_t;
            }

            self.last = Some(value);

            return None;
        }

        self.avg_gain = (self.avg_gain * self.periods_minus_one_as_t + gain) / self.periods_as_t;
        self.avg_loss = (self.avg_loss * self.periods_minus_one_as_t + loss) / self.periods_as_t;

        let rsi: T = if self.avg_loss == self.zero_as_t {
            self.one_hundred_as_t
        } else if self.avg_gain == self.zero_as_t {
            self.zero_as_t
        } else {
            self.one_hundred_as_t * self.avg_gain / (self.avg_gain + self.avg_loss)
        };

        Some(rsi)
    }

    #[inline]
    fn reset(&mut self) {
        self.count = 0;
        self.avg_gain = T::default();
        self.avg_loss = T::default();
        self.sum_gain = T::default();
        self.sum_loss = T::default();
        self.last = None;
    }
}

/// # Average true range
pub struct ATR<T> {
    pub(crate) count: usize,
    pub(crate) atr: Option<T>,
    pub(crate) last_close: Option<T>,
    pub(crate) periods: usize,
    pub(crate) periods_minus_one_t: T,
    pub(crate) periods_t: T,
    pub(crate) tr_accumulator: T,
}

impl<T: Number> ATR<T> {
    pub fn new(periods: usize) -> Self {
        Self {
            count: 0,
            atr: None,
            last_close: None,
            periods,
            periods_minus_one_t: T::from_usize(periods - 1).unwrap(),
            periods_t: T::from_usize(periods).unwrap(),
            tr_accumulator: T::default(),
        }
    }
}

fn get_tr<T: Number>(high: T, low: T, prev_close: T) -> T {
    partial_max2(high, prev_close) - partial_min2(low, prev_close)
}

impl<T: Number> Volatility for ATR<T> {
    type Input = [T; 3]; // high low close
    type Output = Option<T>;

    // original atr definition and this implementation both use wilder’s smoothing
    // wilder’s method has moderate lag. smoothing method should be configurable
    fn push(&mut self, input: Self::Input) -> Self::Output {
        if self.last_close.is_none() {
            self.last_close = Some(input[2]);
            return None;
        }

        let last_close = self.last_close.unwrap();
        self.last_close = Some(input[2]);

        let tr = get_tr(input[0], input[1], last_close);

        if self.atr.is_none() {
            self.tr_accumulator = self.tr_accumulator + tr;
            self.count += 1;

            if self.count == self.periods {
                self.atr = Some(self.tr_accumulator / self.periods_t);

                return self.atr;
            }

            return None;
        }

        let last_atr = self.atr.unwrap();

        self.atr = Some((last_atr * self.periods_minus_one_t + tr) / self.periods_t);

        self.atr
    }

    fn reset(&mut self) {
        self.count = 0;
        self.atr = None;
        self.last_close = None;
        self.tr_accumulator = T::default();
    }
}

/// # Bollinger bands
pub struct BBANDS;
/// # Stochastic oscillator
pub struct STOCH;
/// # Average directional index
pub struct ADX;
/// # Volume weighted average price
pub struct VWAP;

// 2.
/// # Keltner Channels
pub struct KC;
/// # Accumulation/Distribution
pub struct AD;
/// # Chaikin money flow
pub struct CMF;
/// # Ultimate oscillator
pub struct UO;
/// # Triple exponential average
pub struct TRIX;
/// # Ichimoku cloud
pub struct ICHIMOKU;
/// # Aroon
pub struct AROON;
/// # Commodity channel index
pub struct CCI;
/// # Williams %r
pub struct WR;
/// # Rate of change
pub struct ROC;
/// # On balance volume
pub struct OBV;
/// # Money flow index
pub struct MFI;
/// # Parabolic stop and reverse
pub struct SAR;
/// # Donchian channels
pub struct DONCH;

// 3.
/// # Kaufmans adaptive moving average
pub struct KAMA;
/// # Vortex indicator
pub struct VORTEX;
