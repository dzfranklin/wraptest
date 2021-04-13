# wraptest

[![Crates.io](https://img.shields.io/crates/v/wraptest)][crates-io]
[![Crates.io](https://img.shields.io/crates/l/wraptest)][crates-io]

A simple way to run code before and after every unit test.

For example, if you wanted to set up a tracing subscriber before every test:

```rust
#[cfg(test)]
#[wraptest::wrap_tests(before = setup_logs)]
mod tests {
    use tracing::info;
    use tracing_subscriber::fmt::format::FmtSpan;

    fn setup_logs() {
        tracing_subscriber::fmt::fmt()
            .with_env_filter("debug")
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .init();
    }

    #[test]    
    fn with_tracing() {
        info!("with tracing");
    }

    #[tokio::test]
    async fn with_tracing_async() {
        info!("with tracing -- but async");
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
