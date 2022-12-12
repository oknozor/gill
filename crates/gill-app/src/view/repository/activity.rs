use crate::apub::object::repository::RepositoryWrapper;
use crate::apub::object::user::UserWrapper;
use crate::error::AppError;
use crate::get_connected_user;
use crate::oauth::Oauth2User;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::Extension;
use gill_db::repository::Repository;
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

    let repository = Repository::by_namespace(&owner, &repository, &db).await?;

    // Add a 'repository_star' entry to our local db
    repository.add_star(user.id, &db).await?;

    // If the repo is hosted on another instance send a 'Star' activity
    if !repository.is_local {
        let user = UserWrapper::from(user);
        user.star_repository(&RepositoryWrapper::from(repository), &state.instance)
            .await?;
    }

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

    let repository = Repository::by_namespace(&owner, &repository, &db).await?;
    // Add a 'repository_watch' entry to our local db
    repository.add_watcher(user.id, &db).await?;

    // If the repo is hosted on another instance send a 'Watch' activity
    if !repository.is_local {
        let user = UserWrapper::from(user);
        user.watch_repository(&RepositoryWrapper::from(repository), &state.instance)
            .await?;
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}
