#![allow(warnings)]

use crate::strategy::Exponential;
use std::time::Duration;

/// Each retry uses a delay which is the sum of the two previous delays.
///
/// Depending on the problem at hand, a fibonacci delay strategy might perform better and lead to
/// better throughput than the [`Exponential`] strategy.
///
/// See ["A Performance Comparison of Different Backoff Algorithms under Different Rebroadcast
/// Probabilities for MANETs"](https://www.researchgate.net/publication/255672213_A_Performance_Comparison_of_Different_Backoff_Algorithms_under_Different_Rebroadcast_Probabilities_for_MANET's)
/// for more details.
#[derive(Debug)]
pub struct Fibonacci {
    curr: u64,
    next: u64,
}

impl Fibonacci {
    /// Create a new [`Fibonacci`] using the given duration in milliseconds.
    pub fn from_millis(millis: u64) -> Fibonacci {
        Fibonacci {
            curr: millis,
            next: millis,
        }
    }
}

impl Iterator for Fibonacci {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        let duration = Duration::from_millis(self.curr);

        if let Some(next_next) = self.curr.checked_add(self.next) {
            self.curr = self.next;
            self.next = next_next;
        } else {
            self.curr = self.next;
            self.next = u64::MAX;
        }

        Some(duration)
    }
}

impl From<Duration> for Fibonacci {
    fn from(duration: Duration) -> Self {
        Self::from_millis(duration.as_millis() as u64)
    }
}

#[test]
fn fibonacci() {
    let mut iter = Fibonacci::from_millis(10);
    assert_eq!(iter.next(), Some(Duration::from_millis(10)));
    assert_eq!(iter.next(), Some(Duration::from_millis(10)));
    assert_eq!(iter.next(), Some(Duration::from_millis(20)));
    assert_eq!(iter.next(), Some(Duration::from_millis(30)));
    assert_eq!(iter.next(), Some(Duration::from_millis(50)));
    assert_eq!(iter.next(), Some(Duration::from_millis(80)));
}

#[test]
fn fibonacci_saturated() {
    let mut iter = Fibonacci::from_millis(u64::MAX);
    assert_eq!(iter.next(), Some(Duration::from_millis(u64::MAX)));
    assert_eq!(iter.next(), Some(Duration::from_millis(u64::MAX)));
}
