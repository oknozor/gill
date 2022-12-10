use axum::http::header::CONTENT_TYPE;
use axum::http::Request;
use axum::http::StatusCode;
use gill_app::api::user::User;
use serde_json::json;
use speculoos::prelude::*;
use sqlx::PgPool;
use tower::ServiceExt;

use crate::helpers::{service, RequestBuilderExt};

#[sqlx::test]
async fn create_user(db: PgPool) -> anyhow::Result<()> {
    let request = Request::post("/users")
        .header(CONTENT_TYPE, "application/json")
        .json(json! {{ "username": "okno", "email": "ok@no.org" }});

    let response = service(db).oneshot(request).await?;

    assert_that!(response.status()).is_equal_to(StatusCode::NO_CONTENT);

    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn by_email(db: PgPool) -> anyhow::Result<()> {
    let user = User::by_email("alice@wonder.land", &db).await;

    assert_that!(user).is_ok().is_equal_to(User {
        id: 0,
        username: "alice".to_string(),
        email: "alice@wonder.land".to_string(),
    });

    Ok(())
}
