#[cfg(test)]
#[wraptest::wrap_tests(wrapper = with_logs, async_wrapper = with_logs_async)]
mod tests {
    use std::{future::Future, time::Duration};
    use tracing::info;
    use tracing_subscriber::{
        fmt::{
            format::{DefaultFields, FmtSpan, Format},
            TestWriter,
        },
        EnvFilter, FmtSubscriber,
    };

    fn make_subscriber() -> FmtSubscriber<DefaultFields, Format, EnvFilter, TestWriter> {
        tracing_subscriber::fmt::fmt()
            .with_env_filter("debug")
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_test_writer()
            .finish()
    }

    fn with_logs<T, R>(test_fn: T) -> R
    where
        T: FnOnce() -> R,
    {
        tracing::subscriber::with_default(make_subscriber(), test_fn)
    }

    async fn with_logs_async<T, F, R>(test_fn: T) -> R
    where
        T: FnOnce() -> F,
        F: Future<Output = R>,
    {
        let _guard = tracing::subscriber::set_default(make_subscriber());
        test_fn().await
    }

    #[test]
    fn basic() {
        info!("in basic");
    }

    #[tokio::test]
    async fn basic_async() {
        info!("in basic async");
        tokio::time::sleep(Duration::from_millis(10)).await;
        info!("finishing basic async");
    }

    #[test]
    fn returns_result() -> Result<(), String> {
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn with_test_options() {
        info!("with test options");
    }
}
