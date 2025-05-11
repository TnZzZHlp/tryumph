use std::time::Duration;

/// Each retry increases the delay since the last exponentially.
#[derive(Debug)]
pub struct Exponential {
    current: u64,
    factor: f64,
}

impl Exponential {
    /// Create a new [`Exponential`] using the given millisecond duration as the initial delay and
    /// an exponential backoff factor of `2.0`.
    pub fn from_millis(base: u64) -> Self {
        Exponential {
            current: base,
            factor: 2.0,
        }
    }

    /// Create a new [`Exponential`] using the given millisecond duration as the initial delay and
    /// the same duration as the exponential backoff factor. This was the behavior of
    /// [`Exponential::from_millis`] prior to version 2.0.
    pub fn from_millis_with_base_factor(base: u64) -> Self {
        Exponential {
            current: base,
            factor: base as f64,
        }
    }

    /// Create a new [`Exponential`] using the given millisecond duration as the initial delay and
    /// the given exponential backoff factor.
    pub fn from_millis_with_factor(base: u64, factor: f64) -> Self {
        Exponential {
            current: base,
            factor,
        }
    }
}

impl Iterator for Exponential {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        let duration = Duration::from_millis(self.current);

        let next = (self.current as f64) * self.factor;
        self.current = if next > (u64::MAX as f64) {
            u64::MAX
        } else {
            next as u64
        };

        Some(duration)
    }
}

impl From<Duration> for Exponential {
    fn from(duration: Duration) -> Self {
        Self::from_millis(duration.as_millis() as u64)
    }
}

#[test]
fn exponential_with_factor() {
    let mut iter = Exponential::from_millis_with_factor(1000, 2.0);
    assert_eq!(iter.next(), Some(Duration::from_millis(1000)));
    assert_eq!(iter.next(), Some(Duration::from_millis(2000)));
    assert_eq!(iter.next(), Some(Duration::from_millis(4000)));
    assert_eq!(iter.next(), Some(Duration::from_millis(8000)));
    assert_eq!(iter.next(), Some(Duration::from_millis(16000)));
    assert_eq!(iter.next(), Some(Duration::from_millis(32000)));
}

#[test]
fn exponential_overflow() {
    let mut iter = Exponential::from_millis(u64::MAX);
    assert_eq!(iter.next(), Some(Duration::from_millis(u64::MAX)));
    assert_eq!(iter.next(), Some(Duration::from_millis(u64::MAX)));
}
