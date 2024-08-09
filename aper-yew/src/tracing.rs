use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_web::{performance_layer, MakeWebConsoleWriter};

pub fn init_tracing() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .without_time() // std::time is not available in browsers
        .with_writer(MakeWebConsoleWriter::new()); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();
}
