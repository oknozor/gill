use axum::http::header::CONTENT_TYPE;
use axum::http::Request;
use axum::http::StatusCode;
use gill_api::api::repository::OwnedRepository;
use serde_json::json;
use speculoos::prelude::*;
use sqlx::PgPool;
use tower::ServiceExt;

use crate::helpers::{response_json, service, RequestBuilderExt};

#[sqlx::test(fixtures("base"))]
async fn init(db: PgPool) -> anyhow::Result<()> {
    let request = Request::post("/repositories")
        .header(CONTENT_TYPE, "application/json")
        .json(json! {{ "name": "gill" }});

    let response = service(db).oneshot(request).await?;

    assert_that!(response.status()).is_equal_to(StatusCode::NO_CONTENT);
    Ok(())
}

#[sqlx::test(fixtures("base"))]
async fn list(db: PgPool) -> anyhow::Result<()> {
    let request = Request::get("/repositories").empty_body();

    let mut response = service(db).oneshot(request).await?;

    assert_that!(response.status()).is_equal_to(StatusCode::OK);

    let repositories = response_json::<Vec<OwnedRepository>>(&mut response).await;
    let repositories: Vec<(&str, &str)> = repositories
        .iter()
        .map(|repo| (repo.owner_name.as_str(), repo.name.as_str()))
        .collect();

    assert_that!(repositories).contains_all_of(&[
        &("okno", "gill"),
        &("okno", "onagre"),
        &("okno", "cocogitto"),
        &("okno", "linux"),
        &("okno", "postgresql"),
        &("okno", "gitlab"),
        &("alice", "Atalanta"),
        &("alice", "Gaëlle"),
        &("alice", "Damiana"),
        &("alice", "Lugus"),
        &("alice", "Dora"),
        &("alice", "Simon"),
        &("alice", "Knut"),
        &("alice", "Siniša"),
        &("alice", "Yvonne"),
    ]);

    Ok(())
}
