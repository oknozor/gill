use crate::domain::repository::Repository;
use crate::error::AppError;
use crate::get_connected_user;
use crate::oauth::Oauth2User;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::Extension;
use http::StatusCode;
use sqlx::PgPool;

pub async fn star(
    State(state): State<AppState>,
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let Some(user) = get_connected_user(&db, user).await else {
        return Err(AppError::Unauthorized);
    };

    Repository::by_namespace(&owner, &repository, &db)
        .await?
        .add_star(&user, &state.instance)
        .await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

pub async fn watch(
    State(state): State<AppState>,
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let Some(user) = get_connected_user(&db, user).await else {
        return Err(AppError::Unauthorized);
    };

    Repository::by_namespace(&owner, &repository, &db)
        .await?
        .add_watcher(&user, &state.instance)
        .await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
