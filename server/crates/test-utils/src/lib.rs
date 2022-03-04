pub mod db_utils;
use assert_json_diff::assert_json_include;

#[allow(dead_code, clippy::unused_unit)]
pub fn logging_setup() -> () {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{
        fmt::format::{Format, PrettyFields},
        layer::SubscriberExt,
        Registry,
    };

    let logger = tracing_subscriber::fmt::layer()
        .event_format(Format::default().pretty())
        .fmt_fields(PrettyFields::new());

    let subscriber = Registry::default().with(LevelFilter::DEBUG).with(logger);

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing_log::LogTracer::init().expect("logging to work correctly")
}

pub fn assert_200_response(
    response: http::Response<bytes::Bytes>,
    expected_body: serde_json::Value,
) -> serde_json::Value {
    assert_response(
        response,
        http::StatusCode::OK,
        serde_json::json!({
            "status": { "code": 200 },
            "data": expected_body
        }),
    )
}

pub fn assert_200_list_response(
    response: http::Response<bytes::Bytes>,
    expected_body: serde_json::Value,
    total: usize,
    has_more: bool,
) {
    assert_response(
        response,
        http::StatusCode::OK,
        serde_json::json!({
            "status": { "code": 200 },
            "data": expected_body,
            "page": {
                "more": has_more,
                "total": total,
            }
        }),
    );
}

pub fn assert_response(
    response: http::Response<bytes::Bytes>,
    status: http::StatusCode,
    expected_body: serde_json::Value,
) -> serde_json::Value {
    let body = String::from_utf8(response.body().to_vec()).unwrap();
    println!("{:?}", body);
    let parsed_body: serde_json::Value = match serde_json::from_str(&body) {
        Err(e) => {
            println!("Unable to deserialize {:?}. Error: {:?}", body, e);
            unreachable!()
        }
        Ok(body) => body,
    };
    assert_json_include!(actual: parsed_body.clone(), expected: expected_body);
    assert_eq!(response.status(), status);
    parsed_body
}

pub fn assert_error_response(
    response: http::Response<bytes::Bytes>,
    status: http::StatusCode,
    message: &str,
) {
    use json::object;
    let body = String::from_utf8(response.body().to_vec()).unwrap();
    println!("{:?}", body);
    let body = match json::parse(&body) {
        Err(e) => {
            println!("Unable to deserialize {:?}. Error: {:?}", body, e);
            unreachable!()
        }
        Ok(body) => body,
    };
    assert_eq!(
        json::stringify(body),
        json::stringify(object! {
            "status": { "code": response.status().as_u16(), "error": [message] },
        })
    );
    assert_eq!(response.status(), status);
}
