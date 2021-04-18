#[cfg(test)]
#[wraptest::wrap_tests(async_wrapper = with_setup)]
mod tests {
    use std::future::Future;

    async fn with_setup<T, F, R>(test_fn: T) -> R
    where
        T: FnOnce() -> F,
        F: Future<Output = R>,
    {
        eprintln!("before");
        let result = test_fn().await;
        eprintln!("after");
        result
    }

    #[tokio::test]
    async fn async_only() {
        assert!(true);
    }
}
