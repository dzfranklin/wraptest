#[cfg(test)]
#[wraptest::wrap_tests(wrapper = with_setup)]
mod tests {
    fn with_setup<F, R>(test_fn: F) -> R
    where
        F: FnOnce() -> R,
    {
        eprintln!("before");
        let result = test_fn();
        eprintln!("after");
        result
    }

    #[test]
    fn basic_sync() {
        assert!(true);
    }
}
