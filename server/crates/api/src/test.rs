use crate::prelude::*;
use gadget_backend::prelude::*;
use gadget_test_utils::{db_utils::*, *};
use serde_json::json;
use serial_test::serial;
use std::sync::Arc;
use warp::test::request;
use warp::Filter;

async fn make_backend() -> SharedContext {
    setup_schema().await;
    let backend = DefaultBackend::new(&get_db_url_with_test_db())
        .await
        .unwrap();
    Arc::new(SharedData {
        backend,
        ui_location: url::Url::parse("https://example").unwrap(),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial]
async fn create_redirect() {
    // let _logger = logging_setup();
    let context = make_backend().await;
    let filter =
        crate::endpoints::api_endpoint(context).recover(crate::static_response::handle_rejection);

    let response = request()
        .path("/_gadget/api/redirect")
        .body(
            json!({
                "alias": "google",
                "destination": "http://google.com"
            })
            .to_string(),
        )
        .method("POST")
        .reply(&filter)
        .await;

    let parsed = assert_200_response(
        response,
        json!({"alias":  "google", "destination": "http://google.com", "created_by":{"name":"test user","id":"1234"}}),
    );
    let ref_id = parsed["data"]["public_ref"].as_str().unwrap();

    let response = request()
        .path(&format!("/_gadget/api/redirect/{}", ref_id))
        .body(
            json!({
                "destination": "http://google.com"
            })
            .to_string(),
        )
        .method("PUT")
        .reply(&filter)
        .await;
    assert_200_response(
        response,
        json!({
            "public_ref": parsed["data"]["public_ref"],
            "alias": "google",
            "destination": "http://google.com",
            "created_by":{"name":"test user","id":"1234"}
        }),
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial]
async fn update_redirect() {
    // let _logger = logging_setup();
    let context = make_backend().await;
    let filter =
        crate::endpoints::api_endpoint(context).recover(crate::static_response::handle_rejection);

    let response = request()
        .path("/_gadget/api/redirect")
        .body(
            json!({
                "alias": "google",
                "destination": "http://google.com"
            })
            .to_string(),
        )
        .method("POST")
        .reply(&filter)
        .await;
    let body = String::from_utf8(response.body().to_vec()).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let ref_id = parsed["data"]["public_ref"].as_str().unwrap();

    let response = request()
        .path(&format!("/_gadget/api/redirect/{}", ref_id))
        .body(
            json!({
                "destination": "http://yahoo.com"
            })
            .to_string(),
        )
        .method("PUT")
        .reply(&filter)
        .await;

    assert_200_response(
        response,
        json!({
            "public_ref": parsed["data"]["public_ref"],
            "alias": "google",
            "destination": "http://yahoo.com",
            "created_by":{"name":"test user","id":"1234"}
        }),
    );
}
