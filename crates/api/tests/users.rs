use crate::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::http::Request;
use axum::{http, Router, RouterService};
use serde_json::json;
use server::app;
use server::route::user::User;
use speculoos::prelude::*;
use sqlx::PgPool;
use tower::ServiceExt;

mod common;

use crate::common::{response_json, RequestBuilderExt};

#[sqlx::test]
async fn create_user(db: PgPool) -> eyre::Result<()> {
    let request = Request::post("/users")
        .header(CONTENT_TYPE, "application/json")
        .json(json! {{ "username": "alice" }});

    let response = service(db).oneshot(request).await?;

    assert_that!(response.status()).is_equal_to(StatusCode::NO_CONTENT);
    Ok(())
}

#[sqlx::test(fixtures("users"))]
async fn get_user_by_id(db: PgPool) -> eyre::Result<()> {
    let request = Request::get("/users/1").empty_body();

    let mut response = service(db).oneshot(request).await?;

    assert_that!(response.status()).is_equal_to(StatusCode::OK);
    assert_that!(response_json::<User>(&mut response).await).is_equal_to(User {
        id: 1,
        username: "oknozor".to_string(),
    });
    Ok(())
}

fn service(db: PgPool) -> RouterService {
    let router = app::app(db);
    let router = Router::from(router);
    router.into_service()
}
