use aide::axum::IntoApiResponse;
use aide::openapi::OpenApi;
use axum::{Extension, Json};

pub(crate) async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}
