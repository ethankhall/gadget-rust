use lazy_static::lazy_static;
use prometheus::{Counter, CounterVec, Encoder, HistogramVec, TextEncoder};

use warp::http::header::CONTENT_TYPE;

lazy_static! {
    static ref HTTP_COUNTER: Counter = register_counter!(opts!(
        "http_requests_total",
        "Total number of HTTP requests made.",
        labels! {"handler" => "all",}
    ))
    .unwrap();
    static ref HTTP_RESPONSE_CODES_BY_PATH: CounterVec = register_counter_vec!(
        "http_response_status",
        "The HTTP response response codes.",
        &["code", "path"]
    )
    .unwrap();
    static ref HTTP_REQ_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "The HTTP request latencies in seconds.",
        &["method", "path"]
    )
    .unwrap();
}

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

pub struct Metrics;

impl Default for Metrics {
    fn default() -> Self {
        Metrics {}
    }
}

pub fn track_status(info: warp::filters::log::Info) {
    let status = info.status().as_u16();
    let path = info.path();
    let method = info.method();

    HTTP_COUNTER.inc();
    HTTP_RESPONSE_CODES_BY_PATH
        .with_label_values(&[&status.to_string(), path])
        .inc();
    HTTP_REQ_HISTOGRAM
        .with_label_values(&[&method.as_str(), path])
        .observe(duration_to_seconds(info.elapsed()));
}

fn duration_to_seconds(d: std::time::Duration) -> f64 {
    let nanos = f64::from(d.subsec_nanos()) / 1e9;
    d.as_secs() as f64 + nanos
}
