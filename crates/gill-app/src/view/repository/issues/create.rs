use crate::domain::issue::create::CreateIssueCommand;
use crate::error::AppError;
use crate::get_connected_user;
use crate::oauth::Oauth2User;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::Redirect;
use axum::{Extension, Form};

use gill_authorize_derive::authorized;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize, Debug)]
pub struct CreateIssueForm {
    pub title: String,
    pub content: String,
}

#[authorized]
pub async fn create(
    user: Option<Oauth2User>,
    Path((owner, repository)): Path<(String, String)>,
    State(state): State<AppState>,
    Extension(db): Extension<PgPool>,
    Form(form): Form<CreateIssueForm>,
) -> Result<Redirect, AppError> {
    CreateIssueCommand::from(form)
        .execute(&repository, &owner, user, &state.instance)
        .await?;

    Ok(Redirect::to(&format!("/{owner}/{repository}/issues")))
}
