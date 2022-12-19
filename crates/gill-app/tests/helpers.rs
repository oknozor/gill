use axum::body::{Body, BoxBody, HttpBody};
use axum::http::header::CONTENT_TYPE;
use axum::http::{request, Request};
use axum::response::Response;

use serde::de::DeserializeOwned;

pub trait RequestBuilderExt {
    fn json(self, json: serde_json::Value) -> Request<Body>;
    fn empty_body(self) -> Request<Body>;
}

impl RequestBuilderExt for request::Builder {
    fn json(self, json: serde_json::Value) -> Request<Body> {
        self.header("Content-Type", "application/json")
            .body(Body::from(json.to_string()))
            .expect("failed to build request")
    }

    fn empty_body(self) -> Request<Body> {
        self.body(Body::empty()).expect("failed to build request")
    }
}

pub async fn response_json<T>(resp: &mut Response<BoxBody>) -> T
    where
        T: DeserializeOwned,
{
    assert_eq!(
        resp.headers()
            .get(CONTENT_TYPE)
            .expect("expected Content-Type"), "application/json");

    let body = resp.body_mut();

    let mut bytes = Vec::new();

    while let Some(res) = body.data().await {
        let chunk = res.expect("error reading response body");
        bytes.extend_from_slice(&chunk[..]);
    }

    serde_json::from_slice(&bytes).expect("failed to read response body as json")
}
