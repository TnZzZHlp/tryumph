//! # Tryumph
//!
//! A simple, flexible retry library for Rust that handles operations that may fail.
//!
//! `tryumph` provides mechanisms for executing operations with customizable retry strategies.
//! It supports both synchronous and asynchronous execution models with various delay
//! strategies for retries.
//!
//! ## Features
//!
//! - Synchronous retries through the `sync` module
//! - Asynchronous retries through the `unsync` module
//! - Customizable retry strategies (exponential backoff, fixed interval, no delay, etc.)
//!
//! ## Usage Examples
//!
//! ### Synchronous Usage
//!
//! ```rust
//! use tryumph::sync::retry;
//! use tryumph::strategy::Exponential;
//! use std::time::Duration;
//!
//! // Retry an operation with exponential backoff
//! let result = retry(Exponential::from_millis(100).take(3), || {
//!     // Your fallible operation here
//!     if some_condition() {
//!         Ok("success")
//!     } else {
//!         Err("failure")
//!     }
//! });
//!
//! # fn some_condition() -> bool { true }
//! ```
//!
//! ### Asynchronous Usage
//!
//! ```rust
//! use tryumph::unsync::retry;
//! use tryumph::strategy::Exponential;
//! use std::sync::{Arc, Mutex};
//!
//! async fn example() -> Result<&'static str, &'static str> {
//!     let attempts = Arc::new(Mutex::new(0));
//!     let attempts_clone = attempts.clone();
//!     retry(Exponential::from_millis(3), || {
//!         let value = attempts_clone.clone();
//!         async move {
//!             let mut lock = value.lock().unwrap();
//!             *lock += 1;
//!             if *lock < 2 {
//!                 Err("Error")
//!             } else {
//!                 Ok("Success")
//!             }
//!         }
//!     }).await
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `random`: Enables randomized delay functionality (depends on rand)
//!
//! ## Acknowledgment to the following projects
//! <https://github.com/jimmycuadra/retry>
pub mod strategy;
pub mod sync;
pub mod unsync;
