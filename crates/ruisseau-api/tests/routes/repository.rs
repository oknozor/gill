use axum::http::header::CONTENT_TYPE;
use axum::http::Request;
use axum::http::StatusCode;
use serde_json::json;
use speculoos::prelude::*;
use sqlx::PgPool;
use tower::ServiceExt;

use crate::helpers::{service, RequestBuilderExt};

#[sqlx::test(fixtures("users"))]
async fn init(db: PgPool) -> eyre::Result<()> {
    let request = Request::post("/repositories")
        .header(CONTENT_TYPE, "application/json")
        .json(json! {{ "name": "ruisseau" }});

    let response = service(db).oneshot(request).await?;

    assert_that!(response.status()).is_equal_to(StatusCode::NO_CONTENT);
    Ok(())
}
