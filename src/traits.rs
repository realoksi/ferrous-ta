use core::ops::{Add, Div, Mul, Sub};

pub trait Scalar:
    Copy + From<i32> + Add<Output = Self> + Sub<Output = Self> + Div<Output = Self> + Mul<Output = Self>
{
}
impl<T> Scalar for T where
    T: Copy + From<i32> + Add<Output = T> + Sub<Output = T> + Div<Output = T> + Mul<Output = T>
{
}

pub trait Filter {
    type Input;
    type Output;
    /// Steps the filter forward by a single value.
    fn step(&mut self, value: Self::Input) -> Self::Output;
    /// Resets the internal state to their original values.
    fn reset(&mut self);
}
