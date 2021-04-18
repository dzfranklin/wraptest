#[cfg(test)]
#[wraptest::wrap_tests(wrapper = with_setup, async_wrapper = with_setup_async)]
mod tests {
    use std::{error::Error, future::Future, time::Duration};

    fn with_setup<F, R>(test_fn: F) -> R
    where
        F: FnOnce() -> R,
    {
        eprintln!("before");
        let result = test_fn();
        eprintln!("after");
        result
    }

    async fn with_setup_async<T, F, R>(test_fn: T) -> R
    where
        T: FnOnce() -> F,
        F: Future<Output = R>,
    {
        eprintln!("before");
        let result = test_fn().await;
        eprintln!("after");
        result
    }

    #[test]
    fn basic_sync() {
        eprintln!("basic_sync");
    }

    #[test]
    fn basic_result() -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    #[tokio::test]
    async fn basic_async() {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
