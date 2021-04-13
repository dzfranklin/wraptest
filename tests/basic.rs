use std::time::Duration;
use wraptest::wraptest;

fn log_before() {
    eprintln!("--before--");
}

fn log_after() {
    eprintln!("--after--");
}

#[wraptest(before=log_before, after=log_after)]
fn basic() {
    eprintln!("in basic");
}

#[wraptest(before=log_before)]
fn basic_only_before() {
    eprintln!("in basic_only_before");
}

#[wraptest(before = log_before, after = log_after)]
async fn basic_async() {
    eprintln!("in basic async");
    tokio::time::sleep(Duration::from_millis(10)).await;
    eprintln!("finishing basic async");
}
