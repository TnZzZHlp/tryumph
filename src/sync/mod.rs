//! Synchronous retry mechanism for operations that may fail, with customizable delay strategies.
use std::{thread::sleep, time::Duration};

/// Execute an operation and retry it with specified delay intervals if it fails.
///
/// This function repeatedly executes the provided operation until it succeeds or there are no more
/// retry attempts remaining. Before each retry, it waits for the duration provided by the next
/// element from the iterable.
///
/// # Parameters
///
/// * `iterable` - An iterable that provides the durations to wait between retries.
/// * `operation` - The operation to execute, typically a closure that returns a value convertible to `Result`.
///
/// # Returns
///
/// If the operation eventually succeeds, returns `Ok(R)`. If all retries fail, returns the last error `Err(E)`.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use tryumph::sync::retry;
/// use tryumph::strategy::Exponential;
///
/// // Try to get data from a potentially failing API
/// let result = retry(Exponential::from_millis(100).take(3), || {
///     // Your operation that may fail
///     let response = make_api_request();
///     if response.is_success() {
///         Ok(response.data)
///     } else {
///         Err(response.error)
///     }
/// });
///
/// match result {
///     Ok(data) => println!("Successfully retrieved data: {:?}", data),
///     Err(e) => println!("Failed after multiple retries: {:?}", e),
/// }
/// # fn make_api_request() -> Response { Response { is_success: true, data: "data", error: "error" } }
/// # struct Response { is_success: bool, data: &'static str, error: &'static str }
/// # impl Response { fn is_success(&self) -> bool { self.is_success } }
/// ```
///
/// You can use different retry strategies:
///
/// ```
/// use tryumph::sync::retry;
/// use tryumph::strategy::{Fixed, NoDelay};
///
/// // Retry with fixed intervals
/// let result_fixed = retry(Fixed::from_millis(100).take(5), || {
///     // Your operation
///     Ok::<_, &str>("success")
/// });
///
/// // Retry immediately without delays
/// let result_nodelay = retry(NoDelay.take(3), || {
///     // Your operation
///     Ok::<_, &str>("success")
/// });
/// ```
pub fn retry<I, OP, R, O, E>(iterable: I, mut operation: OP) -> Result<O, E>
where
    I: IntoIterator<Item = Duration>,
    OP: FnMut() -> R,
    R: Into<Result<O, E>>,
{
    let mut iter = iterable.into_iter();

    loop {
        // Invoke the factory to obtain a new Result for this attempt.
        match operation().into() {
            Ok(result) => return Ok(result),
            Err(err) => {
                if let Some(duration) = iter.next() {
                    sleep(duration);
                } else {
                    return Err(err); // No more retries left; returning the last error
                }
            }
        }
    }
}

/// Executes an operation and retries it with a specified callback and delay intervals if it fails.
///
/// **Stability: This API is unstable and may change in future versions.**
///
/// This function repeatedly executes the provided operation until it succeeds, the callback
/// indicates to stop, or there are no more retry attempts remaining. Before each retry,
/// it waits for the duration provided by the next element from the iterable.
/// If the operation fails, the callback function is invoked with the `Result` of the current attempt.
///
/// # Parameters
///
/// * `iterable` - An iterable that provides the `Duration` to wait between retries.
/// * `operation` - The operation to execute, typically a closure that returns a value convertible to `Result<O, E>`.
/// * `callback` - A closure invoked after each failed operation attempt. It receives the `Result<O, E>`
///                of the current attempt as an argument.
///                If the callback returns `true`, retries are stopped, and the current error is returned immediately.
///                If the callback returns `false`, retrying continues (if attempts remain).
///
/// # Returns
///
/// Returns `Ok(O)` if the operation eventually succeeds.
/// Returns the last encountered error `Err(E)` if all retries fail or if the callback indicates to stop.
///
/// # Examples
///
/// ```
/// use tryumph::sync::retry_with_callback;
/// use tryumph::strategy::NoDelay;
/// let mut collection = vec![1, 2].into_iter();
/// let mut callback_called = false;

/// let value = retry_with_callback(
///     NoDelay.take(2),
///     || match collection.next() {
///         Some(n) if n == 2 => Ok(n),
///         Some(_) => Err("not 2"),
///         None => Err("not 2"),
///     },
///     |result| {
///         if let Err(e) = result {
///             callback_called = true;
///             assert_eq!(e, "not 2");
///         }
///         false // Continue retrying
///     },
/// )
/// .unwrap();

/// assert_eq!(value, 2);
/// assert_eq!(callback_called, true);
/// ```
pub fn retry_with_callback<I, OP, C, R, O, E>(
    iterable: I,
    mut operation: OP,
    mut callback: C,
) -> Result<O, E>
where
    I: IntoIterator<Item = Duration>,
    OP: FnMut() -> R,
    C: FnMut(Result<O, E>) -> bool,
    E: Clone,
    R: Into<Result<O, E>>,
{
    let mut iter = iterable.into_iter();

    loop {
        // Invoke the factory to obtain a new Result for this attempt.
        match operation().into() {
            Ok(result) => return Ok(result),
            Err(err) => {
                if let Some(duration) = iter.next() {
                    // Call the callback with the error
                    if callback(Err(err.clone())) {
                        // If the callback returns true, we stop retrying
                        return Err(err);
                    }

                    sleep(duration);
                } else {
                    return Err(err); // No more retries left; returning the last error
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::retry;
    use crate::{
        strategy::{Exponential, Fixed, NoDelay},
        sync::retry_with_callback,
    };

    #[test]
    fn succeeds_with_infinite_retries() {
        let mut collection = vec![1, 2, 3, 4, 5].into_iter();

        let value = retry(NoDelay, || match collection.next() {
            Some(n) if n == 5 => Ok(n),
            Some(_) => Err("not 5"),
            None => Err("not 5"),
        })
        .unwrap();

        assert_eq!(value, 5);
    }

    #[test]
    fn succeeds_with_maximum_retries() {
        let mut collection = vec![1, 2].into_iter();

        let value = retry(NoDelay.take(1), || match collection.next() {
            Some(n) if n == 2 => Ok(n),
            Some(_) => Err("not 2"),
            None => Err("not 2"),
        })
        .unwrap();

        assert_eq!(value, 2);
    }

    #[test]
    fn fails_after_last_try() {
        let mut collection = vec![1].into_iter();

        let res = retry(NoDelay.take(1), || match collection.next() {
            Some(n) if n == 2 => Ok(n),
            Some(_) => Err("not 2"),
            None => Err("not 2"),
        });

        assert_eq!(res, Err("not 2"));
    }

    #[test]
    fn fatal_errors() {
        let mut collection = vec![1].into_iter();

        let res = retry(NoDelay.take(2), || match collection.next() {
            Some(n) if n == 2 => Ok(n),
            Some(_) => Err("no retry"),
            None => Err("not 2"),
        });

        assert_eq!(res, Err("not 2"));
    }

    #[test]
    fn succeeds_with_fixed_strategy() {
        let mut collection = vec![1, 2].into_iter();

        let value = retry(Fixed::from_millis(1), || match collection.next() {
            Some(n) if n == 2 => Ok(n),
            Some(_) => Err("not 2"),
            None => Err("not 2"),
        })
        .unwrap();

        assert_eq!(value, 2);
    }

    #[test]
    fn fixed_strategy_from_duration() {
        assert_eq!(
            Fixed::from_millis(1_000).next(),
            Fixed::from(Duration::from_secs(1)).next(),
        );
    }

    #[test]
    fn succeeds_with_exponential_strategy() {
        let mut collection = vec![1, 2].into_iter();

        let value = retry(Exponential::from_millis(1), || match collection.next() {
            Some(n) if n == 2 => Ok(n),
            Some(_) => Err("not 2"),
            None => Err("not 2"),
        })
        .unwrap();

        assert_eq!(value, 2);
    }

    #[test]
    fn succeeds_with_exponential_strategy_with_factor() {
        let mut collection = vec![1, 2].into_iter();

        let value = retry(
            Exponential::from_millis_with_factor(1000, 2.0),
            || match collection.next() {
                Some(n) if n == 2 => Ok(n),
                Some(_) => Err("not 2"),
                None => Err("not 2"),
            },
        )
        .unwrap();

        assert_eq!(value, 2);
    }

    #[test]
    fn retry_with_callback_test() {
        let mut collection = vec![1, 2].into_iter();
        let mut callback_called = false;

        let value = retry_with_callback(
            NoDelay.take(2),
            || match collection.next() {
                Some(n) if n == 2 => Ok(n),
                Some(_) => Err("not 2"),
                None => Err("not 2"),
            },
            |result| {
                if let Err(e) = result {
                    callback_called = true;
                    assert_eq!(e, "not 2");
                }
                false // Continue retrying
            },
        )
        .unwrap();

        assert_eq!(value, 2);
        assert_eq!(callback_called, true);
    }

    #[test]
    #[cfg(feature = "random")]
    fn succeeds_with_ranged_strategy() {
        use crate::strategy::Range;

        let mut collection = vec![1, 2].into_iter();

        let value = retry(Range::from_millis_exclusive(1, 10), || {
            match collection.next() {
                Some(n) if n == 2 => Ok(n),
                Some(_) => Err("not 2"),
                None => Err("not 2"),
            }
        })
        .unwrap();

        assert_eq!(value, 2);
    }
}
