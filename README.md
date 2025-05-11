# Tryumph

[![Crates.io](https://img.shields.io/crates/v/tryumph.svg)](https://crates.io/crates/tryumph)
[![Documentation](https://docs.rs/tryumph/badge.svg)](https://docs.rs/tryumph)
[![License](https://img.shields.io/crates/l/tryumph.svg)](https://github.com/tnzzzhlp/tryumph#license)
[![CI](https://github.com/tnzzzhlp/tryumph/workflows/CI/badge.svg)](https://github.com/tnzzzhlp/tryumph/actions)

A simple, flexible retry library for Rust that handles operations that may fail.

`tryumph` provides mechanisms for executing operations with customizable retry strategies. It supports both synchronous and asynchronous execution models with various delay strategies for retries.

## Features

- Synchronous retries through the `sync` module
- Asynchronous retries through the `unsync` module (requires the `unsync` feature)
- Various retry delay strategies:
  - Fixed interval
  - Exponential backoff
  - Immediate (no delay)
  - Random range (requires the `random` feature)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tryumph = "0.1.0"
```

To enable asynchronous support:

```toml
[dependencies]
tryumph = { version = "0.1.0", features = ["unsync"] }
```

To enable both async and random delay:

```toml
[dependencies]
tryumph = { version = "0.1.0", features = ["unsync", "random"] }
```

## Usage

### Synchronous Example

```rust
use tryumph::sync::retry;
use tryumph::strategy::Exponential;
use std::time::Duration;

fn main() {
    // Try to get data from a potentially failing API with exponential backoff
    let result = retry(Exponential::from_millis(100).take(3), || {
        // Your operation that may fail
        let response = make_api_request();

        if response.is_success() {
            Ok(response.data)
        } else {
            Err(response.error)
        }
    });

    match result {
        Ok(data) => println!("Successfully retrieved data: {:?}", data),
        Err(e) => println!("Failed after multiple retries: {:?}", e),
    }
}

// Mock functions for example
fn make_api_request() -> Response {
    Response { is_success: true, data: "data", error: "error" }
}
struct Response { is_success: bool, data: &'static str, error: &'static str }
impl Response { fn is_success(&self) -> bool { self.is_success } }
```

### Asynchronous Example

With the `unsync` feature enabled:

```rust
use tryumph::unsync::retry;
use tryumph::strategy::Exponential;

async fn fetch_data() -> Result<String, String> {
    // Try to get data from a potentially failing API
    let result = retry(Exponential::from_millis(100).take(3), || async {
        // Your async operation that may fail
        let response = make_api_request().await;

        if response.is_success() {
            Ok(response.data.to_string())
        } else {
            Err(response.error.to_string())
        }
    }).await;

    match result {
        Ok(data) => {
            println!("Successfully retrieved data: {}", data);
            Ok(data)
        }
        Err(e) => {
            println!("Failed after multiple retries: {}", e);
            Err(e)
        }
    }
}
```

### Using Different Retry Strategies

```rust
use tryumph::sync::retry;
use tryumph::strategy::{Fixed, NoDelay};

// Retry with fixed intervals
let result_fixed = retry(Fixed::from_millis(100).take(5), || {
    // Your operation
    Ok::<_, &str>("success")
});

// Retry immediately without delays
let result_nodelay = retry(NoDelay.take(3), || {
    // Your operation
    Ok::<_, &str>("success")
});
```

## Feature Flags

- `unsync`: Enables asynchronous retry functionality (depends on tokio)
- `random`: Enables randomized delay functionality (depends on rand)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgments

- [retry](https://github.com/jimmycuadra/retry)
