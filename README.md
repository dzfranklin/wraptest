# wraptest

[![Version 0.2.0](https://img.shields.io/crates/v/wraptest)][crates-io]
[![License MIT](https://img.shields.io/crates/l/wraptest)][crates-io]

A simple way to run code before or after every unit test.

The wrapper function you specify is called with each of your tests. In the
wrapper you do any setup you want, call the test function you were provided,
and then do any cleanup.

## Examples

### Basic

Suppose you want to set up a tracing subscriber to display log and tracing
events before some tests:

```rust
#[wraptest::wrap_tests(wrapper = with_logs)]
mod tests {
    use tracing::info;
    use tracing_subscriber::fmt::format::FmtSpan;

    fn with_logs<T>(test_fn: T)
    where T: FnOnce() -> () {
        let subscriber = tracing_subscriber::fmt::fmt()
           .with_env_filter("debug")
           .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
           .with_test_writer()
           .finish();
        let _guard = tracing::subscriber::set_default(subscriber);
        test_fn();
    }

    #[test]
    fn with_tracing() {
        info!("with tracing!");
    }
}
```

### Async

If you have async tests (currently only [`tokio::test`] is supported) you
can provide an async wrapper.

```rust
#[wraptest::wrap_tests(async_wrapper = with_logs)]
mod tests {
    async fn with_logs<T, F>(test_fn: T)
    where
        T: FnOnce() -> F,
        F: Future<Output = ()>,
    {
        let subscriber = /* ... */
        let _guard = tracing::subscriber::set_default(subscriber);
        test_fn();
    }

    #[tokio::test]
    async fn with_tracing() {
        info!("with tracing, but async!");
    }
}
```

### Custom return type

If you want to return something other than `()` from your tests you just
need to change the signature of your wrapper. Here's how you can make your
wrappers generic over any return type:

```rust
#[wraptest::wrap_tests(wrapper = with_setup, async_wrapper = with_setup_async)]
mod tests {
    # use std::{future::Future, time::Duration};

    fn with_setup<T, R>(test_fn: T) -> R
    where
        T: FnOnce() -> R,
    {
        eprintln!("Setting up...");
        let result = test_fn();
        eprintln!("Cleaning up...");
        result
    }

    async fn with_setup_async<T, F, R>(test_fn: T) -> R
    where
        T: FnOnce() -> F,
        F: Future<Output = R>,
    {
        eprintln!("Setting up...");
        let result = test_fn().await;
        eprintln!("Cleaning up...");
        result
    }
}
```

## Alternatives

- [minimal-fixtures][minimal-fixtures]
- [rstest][rstest]
- [test-env-log][test-env-log]

I want to especially thank d-e-s-o for test-env-log. If I hadn't seen it, using
macros to reduce redundant test setup wouldn't have occurred to me.

[minimal-fixtures]: https://github.com/vorner/minimal-fixtures
[rstest]: https://github.com/la10736/rstest
[test-env-log]: https://github.com/d-e-s-o/test-env-log
[crates-io]: https://crates.io/crates/wraptest
