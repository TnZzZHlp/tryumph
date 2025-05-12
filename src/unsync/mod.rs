//! Asynchronous retry mechanism for operations that may fail, with customizable delay strategies.
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Execute an asynchronous operation and retry it with specified delay intervals if it fails.
///
/// This function repeatedly executes the provided asynchronous operation until it succeeds
/// or there are no more retry attempts remaining. Before each retry, it waits for the
/// duration provided by the next element from the iterable.
///
/// # Parameters
///
/// * `iterable` - An iterable that provides the durations to wait between retries.
/// * `operation` - The operation to execute, typically a closure that returns a `Future`
///   which resolves to a value convertible to `Result`.
///
/// # Returns
///
/// If the operation eventually succeeds, returns `Ok(R)`. If all retries fail, returns
/// the last error `Err(E)`.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use tryumph::unsync::retry;
/// use tryumph::strategy::Exponential;
///
/// # async fn make_api_request() -> Response { Response { is_success: true, data: "data", error: "error" } }
/// # struct Response { is_success: bool, data: &'static str, error: &'static str }
/// # impl Response { fn is_success(&self) -> bool { self.is_success } }
///
/// async fn fetch_data() -> Result<String, String> {
///     // Try to get data from a potentially failing API
///     let result = retry(Exponential::from_millis(100).take(3), || async {
///         // Your async operation that may fail
///         let response = make_api_request().await;
///         if response.is_success() {
///             Ok(response.data.to_string())
///         } else {
///             Err(response.error.to_string())
///         }
///     }).await;
///
///     match result {
///         Ok(data) => {
///             println!("Successfully retrieved data: {}", data);
///             Ok(data)
///         }
///         Err(e) => {
///             println!("Failed after multiple retries: {}", e);
///             Err(e)
///         }
///     }
/// }
/// ```
///
/// You can use different retry strategies:
///
/// ```
/// use tryumph::unsync::retry;
/// use tryumph::strategy::{Fixed, NoDelay};
///
/// async fn example() {
///     // Retry with fixed intervals
///     let result_fixed = retry(Fixed::from_millis(100).take(5), || async {
///         // Your async operation
///         Ok::<_, &str>("success")
///     }).await;
///
///     // Retry immediately without delays
///     let result_nodelay = retry(NoDelay.take(3), || async {
///         // Your async operation
///         Ok::<_, &str>("success")
///     }).await;
/// }
/// ```
pub async fn retry<I, OP, F, R, O, E>(iterable: I, mut operation: OP) -> Result<O, E>
where
    I: IntoIterator<Item = Duration>,
    OP: FnMut() -> F,
    F: Future<Output = R>,
    R: Into<Result<O, E>>,
{
    let mut iter = iterable.into_iter();

    loop {
        // Invoke the factory to obtain a new Future for this attempt.
        let future_to_await = operation();
        match future_to_await.await.into() {
            Ok(result) => return Ok(result),
            Err(err) => {
                if let Some(duration) = iter.next() {
                    sleep(duration).await;
                } else {
                    return Err(err); // No more retries left; returning the last error
                }
            }
        }
    }
}

/// Executes an asynchronous operation and retries it with a specified asynchronous callback and delay intervals if it fails.
///
/// **Stability: This API is unstable and may change in future versions.**
///
/// This function repeatedly executes the provided asynchronous operation until it succeeds,
/// the asynchronous callback indicates to stop, or there are no more retry attempts remaining.
/// Before each retry, it asynchronously waits for the duration provided by the next element
/// from the iterable. If the operation fails, the asynchronous callback function is invoked
/// with the `Result` of the current attempt.
///
/// # Parameters
///
/// * `iterable` - An iterable that provides the `Duration` to wait between retries.
/// * `operation` - The operation to execute, typically a closure that returns a `Future`
///   which resolves to a value convertible to `Result<O, E>`.
/// * `callback` - An asynchronous closure invoked after each failed operation attempt.
///   It receives the `Result<O, E>` of the current attempt as an argument and
///   returns a `Future` that resolves to a boolean.
///   If the callback's `Future` resolves to `true`, retries are stopped, and the current error is returned immediately.
///   If it resolves to `false`, retrying continues (if attempts remain).
///
/// # Returns
///
/// Returns `Ok(O)` if the operation eventually succeeds.
/// Returns the last encountered error `Err(E)` if all retries fail or if the callback indicates to stop.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use tryumph::unsync::retry_with_callback;
/// use tryumph::strategy::Fixed;
/// use std::sync::{Arc, Mutex}; // Add this for Arc<Mutex<T>>
///
/// # async fn make_api_request_flaky(attempt: u32) -> Result<&'static str, &'static str> {
/// #     if attempt < 3 { Err("network error") } else { Ok("success data") }
/// # }
///
/// async fn fetch_data_with_callback() -> Result<&'static str, &'static str> {
///     // Use Arc<Mutex<u32>> to share and mutate the attempts counter safely
///     let attempts = Arc::new(Mutex::new(0u32));
///
///     retry_with_callback(
///         Fixed::from_millis(50).take(5), // Retry up to 5 times, with 50ms interval
///         || { // Operation
///             // Clone Arc for the operation closure
///             let attempts_clone_op = Arc::clone(&attempts);
///             async move {
///                 let current_attempt = {
///                     let mut attempts_guard = attempts_clone_op.lock().unwrap();
///                     *attempts_guard += 1;
///                     *attempts_guard // Get the current attempt count
///                 };
///                 println!("Operation: Attempt {}", current_attempt);
///                 make_api_request_flaky(current_attempt).await
///             }
///         },
///         |attempt_result| { // Asynchronous Callback
///             // Clone Arc for the callback closure
///             let attempts_clone_cb = Arc::clone(&attempts);
///             async move {
///                 let current_attempt_for_cb = {
///                     // Lock to read the current attempt count
///                     *attempts_clone_cb.lock().unwrap()
///                 };
///                 match attempt_result {
///                     Ok(_) => {
///                         // This arm is unlikely to be hit as callback is called on error.
///                         println!("Callback: Operation succeeded on attempt {} (should not happen here).", current_attempt_for_cb);
///                         false // Continue (though it already succeeded)
///                     }
///                     Err(e) => {
///                         println!("Callback: Operation failed on attempt {} with '{}'.", current_attempt_for_cb, e);
///                         if current_attempt_for_cb >= 3 && e == "network error" {
///                             println!("Callback: Deciding to stop retrying after {} attempts.", current_attempt_for_cb);
///                             true // Stop retrying
///                         } else {
///                             println!("Callback: Will retry.");
///                             false // Continue retrying
///                         }
///                     }
///                 }
///             }
///         },
///     )
///     .await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     match fetch_data_with_callback().await {
///         Ok(data) => println!("Final result: Successfully retrieved data: {:?}", data),
///         Err(e) => println!("Final result: Failed: {:?}", e),
///     }
/// }
/// ```
pub async fn retry_with_callback<I, OP, C, FC, F, R, O, E>(
    iterable: I,
    mut operation: OP,
    mut callback: C,
) -> Result<O, E>
where
    I: IntoIterator<Item = Duration>,
    OP: FnMut() -> F,
    C: FnMut(Result<O, E>) -> FC,
    FC: Future<Output = bool>,
    F: Future<Output = R>,
    E: Clone,
    R: Into<Result<O, E>>,
{
    let mut iter = iterable.into_iter();

    loop {
        // Invoke the factory to obtain a new Future for this attempt.
        let future_to_await = operation();
        match future_to_await.await.into() {
            Ok(result) => return Ok(result),
            Err(err) => {
                if let Some(duration) = iter.next() {
                    // Call the callback function with the error
                    if callback(Err(err.clone())).await {
                        // If the callback returns true, we stop retrying
                        return Err(err);
                    }

                    sleep(duration).await;
                } else {
                    return Err(err); // No more retries left; returning the last error
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use super::*;
    use crate::strategy::{Exponential, Fixed, NoDelay};

    #[tokio::test]
    async fn test_retry() {
        let attempts = Arc::new(Mutex::new(0));
        let attempts_clone = attempts.clone();
        let result = retry(Exponential::from_millis(3), || {
            let value = attempts_clone.clone();
            async move {
                let mut lock = value.lock().unwrap();
                *lock += 1;
                if *lock < 2 {
                    Err("Error")
                } else {
                    Ok("Success")
                }
            }
        })
        .await;

        assert_eq!(result, Ok("Success"));
        assert_eq!(*attempts.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn succeeds_with_infinite_retries() {
        let mut collection = vec![1, 2, 3, 4, 5].into_iter();

        let value = retry(NoDelay, || {
            let next_val = collection.next();
            async move {
                match next_val {
                    Some(n) if n == 5 => Ok(n),
                    Some(_) => Err("not 5"),
                    None => Err("not 5"),
                }
            }
        })
        .await
        .unwrap();

        assert_eq!(value, 5);
    }

    #[tokio::test]
    async fn succeeds_with_maximum_retries() {
        let mut collection = vec![1, 2].into_iter();

        let value = retry(NoDelay.take(1), || {
            let next_val = collection.next();
            async move {
                match next_val {
                    Some(n) if n == 2 => Ok(n),
                    Some(_) => Err("not 2"),
                    None => Err("not 2"),
                }
            }
        })
        .await
        .unwrap();

        assert_eq!(value, 2);
    }

    #[tokio::test]
    async fn fails_after_last_try() {
        let mut collection = vec![1].into_iter();

        let res = retry(NoDelay.take(1), || {
            let next_val = collection.next();
            async move {
                match next_val {
                    Some(n) if n == 2 => Ok(n),
                    Some(_) => Err("not 2"),
                    None => Err("not 2"),
                }
            }
        })
        .await;

        assert_eq!(res, Err("not 2"));
    }

    #[tokio::test]
    async fn fatal_errors_actually_retries_and_fails() {
        let mut collection = vec![1].into_iter();

        let res = retry(NoDelay.take(2), || {
            let next_val = collection.next();
            async move {
                match next_val {
                    Some(n) if n == 2 => Ok(n),
                    Some(_) => Err("some error"),
                    None => Err("exhausted"),
                }
            }
        })
        .await;

        assert_eq!(res, Err("exhausted"));
    }

    #[tokio::test]
    async fn succeeds_with_fixed_strategy() {
        let mut collection = vec![1, 2].into_iter();

        let value = retry(Fixed::from_millis(1), || {
            let next_val = collection.next();
            async move {
                match next_val {
                    Some(n) if n == 2 => Ok(n),
                    Some(_) => Err("not 2"),
                    None => Err("not 2"),
                }
            }
        })
        .await
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

    #[tokio::test]
    async fn succeeds_with_exponential_strategy() {
        let mut collection = vec![1, 2].into_iter();

        let value = retry(Exponential::from_millis(1), || {
            let next_val = collection.next();
            async move {
                match next_val {
                    Some(n) if n == 2 => Ok(n),
                    Some(_) => Err("not 2"),
                    None => Err("not 2"),
                }
            }
        })
        .await
        .unwrap();

        assert_eq!(value, 2);
    }

    #[tokio::test]
    async fn succeeds_with_exponential_strategy_with_factor() {
        let mut collection = vec![1, 2].into_iter();

        let value = retry(Exponential::from_millis_with_factor(10, 2.0), || {
            let next_val = collection.next();
            async move {
                match next_val {
                    Some(n) if n == 2 => Ok(n),
                    Some(_) => Err("not 2"),
                    None => Err("not 2"),
                }
            }
        })
        .await
        .unwrap();

        assert_eq!(value, 2);
    }

    #[tokio::test]
    async fn retry_with_callback_test() {
        let mut collection = vec![1, 2].into_iter();
        let mut callback_called = false;

        let value = retry_with_callback(
            NoDelay.take(2),
            || {
                let next_val = collection.next();
                async move {
                    match next_val {
                        Some(n) if n == 2 => Ok(n),
                        Some(_) => Err("not 2"),
                        None => Err("not 2"),
                    }
                }
            },
            |result| {
                if let Err(e) = result {
                    callback_called = true;
                    assert_eq!(e, "not 2");
                }
                std::future::ready(false) // Continue retrying
            },
        )
        .await
        .unwrap();

        assert_eq!(value, 2);
        assert!(callback_called);
    }

    #[tokio::test]
    #[cfg(feature = "random")]
    async fn succeeds_with_ranged_strategy() {
        use crate::strategy::Range;

        let mut collection = vec![1, 2].into_iter();

        let value = retry(Range::from_millis_exclusive(1, 10), || {
            let next_val = collection.next();
            async move {
                match next_val {
                    Some(n) if n == 2 => Ok(n),
                    Some(_) => Err("not 2"),
                    None => Err("not 2"),
                }
            }
        })
        .await
        .unwrap();

        assert_eq!(value, 2);
    }
}
