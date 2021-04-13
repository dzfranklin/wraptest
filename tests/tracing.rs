use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;
use wraptest::wraptest;

fn setup_logs() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter("debug")
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .init();
}

#[wraptest(before = setup_logs)]
fn with_tracing() {
    info!("with tracing");
}
