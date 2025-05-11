//! Different types of delay for retryable operations.
#![allow(warnings)]

use std::time::Duration;

/// Each retry strategy is an iterator that yields a [`Duration`] for each retry.
pub mod exponential;
pub use exponential::Exponential;

/// A fixed delay strategy that yields the same duration for each retry.
pub mod fibonacci;
pub use fibonacci::Fibonacci;

/// A fixed delay strategy that yields the same duration for each retry.
pub mod fixed;
pub use fixed::Fixed;

/// A no-delay strategy that yields a [`Duration`] of zero for each retry.
pub mod nodelay;
pub use nodelay::NoDelay;

#[cfg(feature = "random")]
mod random;
#[cfg(feature = "random")]
pub use random::{Range, jitter};
