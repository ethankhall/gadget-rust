use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_histogram_vec, CounterVec, Encoder, HistogramVec, TextEncoder,
};

use warp::http::header::CONTENT_TYPE;

lazy_static! {
    static ref HTTP_RESPONSE_CODES_BY_PATH: CounterVec = register_counter_vec!(
        "http_response_status",
        "The HTTP response response codes.",
        &["method", "path", "status", "generic_status"]
    )
    .unwrap();
    static ref HTTP_REQ_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "The HTTP request latencies in seconds.",
        &["method", "path", "status", "generic_status"]
    )
    .unwrap();
}

#[tracing::instrument]
pub fn metrics_endpoint() -> impl warp::Reply {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Ok(warp::reply::with_header(
        buffer,
        CONTENT_TYPE,
        encoder.format_type(),
    ))
}

#[derive(Default)]
pub struct Metrics;

pub fn track_status(info: warp::filters::log::Info) {
    let status = info.status().as_u16();
    let path = info.path();
    let method = info.method();
    let generic_status = format!("{}xx", status / 100);

    HTTP_RESPONSE_CODES_BY_PATH
        .with_label_values(&[method.as_str(), path, &status.to_string(), &generic_status])
        .inc();
    HTTP_REQ_HISTOGRAM
        .with_label_values(&[method.as_str(), path, &status.to_string(), &generic_status])
        .observe(duration_to_seconds(info.elapsed()));
}

fn duration_to_seconds(d: std::time::Duration) -> f64 {
    let nanos = f64::from(d.subsec_nanos()) / 1e9;
    d.as_secs() as f64 + nanos
}
