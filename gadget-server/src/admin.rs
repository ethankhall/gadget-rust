use std::pin::Pin;
use std::task::{Context, Poll};

use lazy_static::lazy_static;
use prometheus::{Counter, CounterVec, Encoder, HistogramVec, TextEncoder};

use actix_service::{Service, Transform};
use actix_web::{
    dev::{HttpResponseBuilder, ServiceRequest, ServiceResponse},
    http::{StatusCode, header::CONTENT_TYPE},
    Error, HttpResponse,
};

use futures::future::{ok, Ready};
use futures::Future;

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

pub fn metrics_endpoint() -> HttpResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    HttpResponseBuilder::new(StatusCode::OK)
        .set_header(CONTENT_TYPE, encoder.format_type())
        .body(buffer)
}

pub struct Metrics;

impl Default for Metrics {
    fn default() -> Self {
        Metrics {}
    }
}

impl<S, B> Transform<S> for Metrics
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PrometheusMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(PrometheusMiddleware { service })
    }
}

pub struct PrometheusMiddleware<S> {
    service: S,
}

type PinBox<T, E> = Pin<Box<dyn Future<Output = Result<T, E>>>>;

impl<S, B> Service for PrometheusMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = PinBox<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        HTTP_COUNTER.inc();

        let (path, hist) = {
            let method = req.method();
            let path = req.uri().path();

            (
                path.to_string(),
                HTTP_REQ_HISTOGRAM
                    .with_label_values(&[&method.to_string(), path])
                    .start_timer(),
            )
        };

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            hist.observe_duration();
            HTTP_RESPONSE_CODES_BY_PATH
                .with_label_values(&[&res.status().as_u16().to_string(), &path])
                .inc();

            Ok(res)
        })
    }
}
