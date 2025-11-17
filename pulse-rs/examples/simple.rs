use pulse_rs::{log, metrics, trace};
use rand::Rng as _;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let config: pulse_rs::Config = toml::from_str(
        r#"
        service_name = "simple"
        service_version = "0.1"
        uri = "grpc://localhost:4317"

        [log]
        level = "info"

        [metrics]
        export_interval = "1s"

        [trace]
        level = "info"
    "#,
    )
    .unwrap();

    let _session = pulse_rs::init(&config).unwrap();

    log::info!("this is an info log");
    let mut i: i32 = 0;
    loop {
        i += 1;
        foo(i).await;
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

#[trace::instrument]
async fn foo(i: i32) {
    trace::error!("starting function");
    trace::info!(i = i);

    metrics::record!(monotonic_counter.foo = 1_u64);

    let num = rand::thread_rng().gen_range(0..100);
    metrics::record!(gauge.my_num = num);

    log::error!("this is an error log");
    log::trace!("this is a trace log");
    log::info!("this is an info log");
    log::warn!("this is a warn log");
    log::debug!("this is a debug log");
}
