#[cfg(test)]
#[wraptest::wrap_tests(before = log_before, after = log_after)]
mod tests {
    use std::time::Duration;
    fn log_before() {
        eprintln!("--before--");
    }

    fn log_after() {
        eprintln!("--after--");
    }

    #[test]
    fn basic() {
        eprintln!("in basic");
    }

    #[tokio::test]
    async fn basic_async() {
        eprintln!("in basic async");
        tokio::time::sleep(Duration::from_millis(10)).await;
        eprintln!("finishing basic async");
    }

    #[test]
    fn returns_result() -> Result<(), String> {
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn with_test_options() {
        eprintln!("with test options");
    }
}
