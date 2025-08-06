use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, Neg, Sub};

pub trait NumLike:
    Add<Output = Self>
    + AddAssign
    + PartialEq
    + PartialOrd
    + Copy
    + Sub<Output = Self>
    + Neg<Output = Self>
    + Div<Output = Self>
{
}
impl<
        T: Add<Output = Self>
            + AddAssign
            + PartialEq
            + PartialOrd
            + Copy
            + Sub<Output = Self>
            + Neg<Output = Self>
            + Div<Output = Self>,
    > NumLike for T
{
}

/// To represent and generate a sequence of item
/// It is inclusive: $[`start`, `end`]$ with interval of `step`
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Sequence<T>
where
    T: NumLike,
{
    start: T,
    step: T,
    end: T,
    current: Option<T>,
}

impl<T> Sequence<T>
where
    T: NumLike,
{
    pub fn new(start: T, step: T, end: T) -> Self {
        Self {
            start,
            step,
            end,
            current: None,
        }
    }
}

impl<T: NumLike> Iterator for Sequence<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.current = self.current.map_or(Some(self.start), |curr| {
            let next = curr + self.step;
            if self.start <= self.end {
                (next <= self.end).then_some(next)
            } else {
                (next >= self.end).then_some(next)
            }
        });
        self.current
    }
}
